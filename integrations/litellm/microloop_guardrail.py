"""
Microloop Guardrail — Deterministic loop detection for LiteLLM.

Hooks into LiteLLM's async_pre_call_hook to examine tool call history
in every request. If the same tool + arguments repeat within a configurable
window, the call is blocked BEFORE it reaches the LLM — saving API costs.

Usage::

    import litellm
    from microloop_guardrail import MicroloopGuardrail

    litellm.callbacks = [MicroloopGuardrail(max_repeats=3)]
"""

from __future__ import annotations

import hashlib
import json
import logging
from typing import Any, Dict, Optional

try:
    from litellm._logging import verbose_logger
    from litellm.integrations.custom_guardrail import CustomGuardrail
    from litellm.proxy._types import UserAPIKeyAuth
    from litellm.types.guardrails import GuardrailEventHooks
    from litellm.types.utils import CallTypes
    from litellm.caching import DualCache
except ImportError:
    CustomGuardrail = object
    DualCache = object
    UserAPIKeyAuth = object
    CallTypes = object
    GuardrailEventHooks = object
    verbose_logger = logging.getLogger("microloop_guardrail")

logger = logging.getLogger("microloop_guardrail")

# ---------------------------------------------------------------------------
# Exception raised when Microloop detects a loop
# ---------------------------------------------------------------------------

class MicroloopLoopDetected(ValueError):
    """Raised when a tool call trajectory repeats beyond the configured limit."""

    def __init__(self, tool_name, repeat_count, max_repeats, session_id=""):
        self.tool_name = tool_name
        self.repeat_count = repeat_count
        self.max_repeats = max_repeats
        self.session_id = session_id
        msg = (
            f"Microloop: Loop detected on tool '{tool_name}' -- "
            f"seen {repeat_count}x (limit: {max_repeats})"
            + (f" in session '{session_id}'" if session_id else "")
        )
        super().__init__(msg)


# ---------------------------------------------------------------------------
# In-memory call history store — hash-only (CWE-400 mitigation)
# ---------------------------------------------------------------------------
# Stores only fixed-size SHA-256 digests (64 bytes each), never raw JSON.
# A malicious client sending 10MB JSON payloads cannot inflate memory beyond
# the per-entry digest cost.
# ---------------------------------------------------------------------------

class _CallHistory:
    """Tracks tool call trajectory digests per session with bounded memory.

    Memory: each entry is (tool_name: str, hash: str) — 64 bytes per digest
    regardless of original payload size.
    """
    MAX_SESSIONS = 10000

    def __init__(self) -> None:
        self._store: dict[str, list[tuple[str, str]]] = {}

    def append(self, session_id: str, tool_name: str, trajectory_hash: str) -> None:
        # Evict oldest session if we hit the memory cap
        if len(self._store) >= self.MAX_SESSIONS and session_id not in self._store:
            oldest_key = next(iter(self._store))
            self._store.pop(oldest_key, None)
        if session_id not in self._store:
            self._store[session_id] = []
        self._store[session_id].append((tool_name, trajectory_hash))

    def get_recent(self, session_id: str, window: int) -> list[tuple[str, str]]:
        entries = self._store.get(session_id, [])
        return entries[-window:]

    def trim(self, session_id: str, max_len: int) -> None:
        entries = self._store.get(session_id, [])
        if len(entries) > max_len:
            self._store[session_id] = entries[-max_len:]

    def correct_last(self, session_id: str, tool_name: str, trajectory_hash: str) -> None:
        """Replace the hash of the most recent entry for *tool_name*.

        Used when auto-inference discovers new volatile fields: the previous
        call's hash must be retroactively fixed so consistency with the new
        canonical form is maintained.
        """
        entries = self._store.get(session_id)
        if not entries:
            return
        for i in range(len(entries) - 1, -1, -1):
            if entries[i][0] == tool_name:
                entries[i] = (tool_name, trajectory_hash)
                return

    def clear(self, session_id: str) -> None:
        self._store.pop(session_id, None)


# ---------------------------------------------------------------------------
# Volatile field utilities
# ---------------------------------------------------------------------------

def _strip_volatile_fields(args_json, volatile_fields):
    """Remove known volatile fields from a JSON arguments string."""
    if not volatile_fields:
        return args_json
    try:
        obj = json.loads(args_json)
        if not isinstance(obj, dict):
            return args_json
        for field in volatile_fields:
            obj.pop(field, None)
        return json.dumps(obj, sort_keys=True, separators=(",", ":"))
    except (json.JSONDecodeError, TypeError):
        return args_json


def _compute_trajectory_hash(tool_name: str, canonical_args: str) -> str:
    """Deterministic SHA-256 digest of a tool call trajectory.

    Returns a hex string (64 bytes) — fixed-size regardless of payload size.
    """
    raw = f"{tool_name}|{canonical_args}"
    return hashlib.sha256(raw.encode()).hexdigest()


def _auto_infer_volatile(
        last_raw_args: Optional[Dict[str, Any]],
        current_args: str) -> set:
    """Detect fields that differ between the last call and current call.

    A field is considered volatile only if:
    1. It is the *only* differing field between the two calls
    2. AND at least one other field stays constant

    This prevents inferring the only changing parameter as volatile.

    Unlike the previous implementation, this only compares the immediately
    preceding raw args (O(1) memory) rather than scanning the full history
    window (O(n) memory), eliminating the unbounded raw-JSON storage vector.
    """
    inferred: set = set()

    if last_raw_args is None:
        return inferred

    try:
        current = json.loads(current_args)
        if not isinstance(current, dict):
            return inferred
    except (json.JSONDecodeError, TypeError):
        return inferred

    if not isinstance(last_raw_args, dict):
        return inferred

    diffs = []
    consts = []
    all_keys = set(current.keys()) | set(last_raw_args.keys())
    for k in all_keys:
        if current.get(k) != last_raw_args.get(k):
            diffs.append(k)
        else:
            consts.append(k)

    if len(diffs) == 1 and consts:
        inferred.add(diffs[0])

    return inferred


# ---------------------------------------------------------------------------
# Main guardrail class
# ---------------------------------------------------------------------------

class MicroloopGuardrail(CustomGuardrail):
    """
    LiteLLM guardrail that detects deterministic tool call loops.

    Intercepts *before* the LLM API call by examining the message history
    included in every request. If the same tool with the same arguments has
    repeated ``max_repeats`` times within the ``history_window``, the call
    is blocked and a ``MicroloopLoopDetected`` exception is raised.

    Memory-safe: all trajectory data is stored as fixed-size SHA-256 digests
    (64 bytes each). Raw JSON arguments are never persisted in the history
    store, preventing CWE-400 (Uncontrolled Resource Consumption) attacks
    via oversized payloads.
    """

    def __init__(
        self,
        max_repeats=3,
        history_window=None,
        volatile_fields=None,
        auto_infer_volatile=True,
        **kwargs,
    ):
        super().__init__(**kwargs)
        self._max_repeats = max_repeats
        self._history_window = history_window or (max_repeats * 2)
        self._volatile_fields = set(volatile_fields or [])
        self._auto_infer_volatile = auto_infer_volatile
        self._history = _CallHistory()
        # O(1) raw-args cache for auto_infer_volatile — only the immediately
        # preceding call per (session, tool), not the full history window.
        self._last_raw_args: dict[str, Dict[str, Any]] = {}

    # ---- LiteLLM hooks ---------------------------------------------------

    async def async_pre_call_hook(
        self,
        user_api_key_dict=None,
        cache=None,
        data=None,
        call_type=None,
    ):
        """
        LiteLLM hook called before each LLM API request.

        Examines ``data["messages"]`` for tool call history and raises
        ``MicroloopLoopDetected`` if a loop is found.
        """
        if data is None:
            data = {}

        session_id = self._get_session_id(data)
        messages = data.get("messages", [])
        tool_pairs = self._extract_tool_calls(messages)

        if not tool_pairs:
            return None

        for tool_name, tool_args in tool_pairs:
            self._check_and_record(session_id, tool_name, tool_args)

        return None  # Allow the call to proceed

    # ---- Core loop detection ---------------------------------------------

    def _check_and_record(self, session_id, tool_name, raw_args):
        """Check if *tool_name(raw_args)* is a loop, then record it.

        Memory safety: raw_args is canonicalized and hashed *before* any
        history interaction. The raw payload is never stored — only the
        fixed-size digest enters _CallHistory.
        """
        # 1. Strip configured volatile fields → canonical args
        canonical = _strip_volatile_fields(raw_args, self._volatile_fields)

        # 2. Auto-inference: compare current raw args to the immediately
        #    preceding raw args (O(1) per-session per-tool cache).
        inferred: set = set()
        if self._auto_infer_volatile:
            cache_key = f"{session_id}:{tool_name}"
            prev_raw = self._last_raw_args.get(cache_key)
            inferred = _auto_infer_volatile(prev_raw, raw_args)
            if inferred:
                combined = self._volatile_fields | inferred
                canonical = _strip_volatile_fields(raw_args, combined)

        # 3. Compute fixed-size digest — raw JSON is NEVER stored after this point
        trajectory_hash = _compute_trajectory_hash(tool_name, canonical)

        # 4. If auto-inference discovered new volatile fields, retroactively fix
        #    the last history entry for this tool so previous hashes match the
        #    new canonical form. This ensures loop detection works across the
        #    boundary where the volatile field was first learned.
        if inferred:
            self._history.correct_last(session_id, tool_name, trajectory_hash)

        # 5. Check digest-only history for repeats
        recent = self._history.get_recent(session_id, self._history_window)
        match_count = sum(
            1 for t, h in recent
            if t == tool_name and h == trajectory_hash
        )

        # 6. Block or record
        if match_count + 1 >= self._max_repeats:
            raise MicroloopLoopDetected(
                tool_name=tool_name,
                repeat_count=match_count + 1,
                max_repeats=self._max_repeats,
                session_id=session_id,
            )

        self._history.append(session_id, tool_name, trajectory_hash)
        self._history.trim(session_id, self._history_window * 2)

        # 7. Update O(1) raw-args cache for next auto-inference
        try:
            parsed = json.loads(raw_args)
            if isinstance(parsed, dict):
                cache_key = f"{session_id}:{tool_name}"
                self._last_raw_args[cache_key] = parsed
                # Cap the cache at MAX_SESSIONS to match _CallHistory
                if len(self._last_raw_args) > _CallHistory.MAX_SESSIONS:
                    self._last_raw_args.pop(next(iter(self._last_raw_args)), None)
        except (json.JSONDecodeError, TypeError):
            pass

    # ---- Helpers ---------------------------------------------------------

    @staticmethod
    def _get_session_id(data):
        session_id = data.get("litellm_session_id")
        if session_id:
            return str(session_id)
        metadata = data.get("metadata", {}) or {}
        session_id = metadata.get("session_id")
        if session_id:
            return str(session_id)
        return "default"

    @staticmethod
    def _extract_tool_calls(messages):
        """Extract (tool_name, arguments_json) pairs from assistant messages.

        Supports both OpenAI ``tool_calls`` and Anthropic ``tool_use`` formats.
        """
        if not isinstance(messages, list):
            return []

        pairs = []
        for msg in reversed(messages):
            if not isinstance(msg, dict):
                continue
            role = msg.get("role", "")
            if role != "assistant":
                continue

            # OpenAI format
            tool_calls = msg.get("tool_calls")
            if isinstance(tool_calls, list):
                for tc in tool_calls:
                    if not isinstance(tc, dict):
                        continue
                    func = tc.get("function", {})
                    name = func.get("name", "") if isinstance(func, dict) else ""
                    args_raw = func.get("arguments", "{}") if isinstance(func, dict) else "{}"
                    if name:
                        pairs.append((name, args_raw))
                if pairs:
                    break

            # Anthropic format
            content = msg.get("content")
            if isinstance(content, list):
                for block in content:
                    if isinstance(block, dict) and block.get("type") == "tool_use":
                        name = block.get("name", "")
                        args_raw = json.dumps(block.get("input", {}), sort_keys=True)
                        if name:
                            pairs.append((name, args_raw))
                if pairs:
                    break

        return pairs
