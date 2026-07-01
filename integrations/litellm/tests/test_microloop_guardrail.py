"""
Tests for MicroloopGuardrail - standalone (no LiteLLM dependency required).

Run with pytest (preferred) or standalone python.

    pytest integrations/litellm/tests/test_microloop_guardrail.py -v
"""

import asyncio
import json
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

import pytest
from microloop_guardrail import (
    MicroloopGuardrail,
    MicroloopLoopDetected,
    _CallHistory,
    _strip_volatile_fields,
    _auto_infer_volatile,
    _compute_trajectory_hash,
)


# ==========================================================================
# Unit: Volatile field utilities
# ==========================================================================

class TestStripVolatileFields:
    def test_strips_single_field(self):
        r = _strip_volatile_fields('{"q":"hello","rid":1}', {'rid'})
        assert 'rid' not in r
        assert 'hello' in r

    def test_passthrough_empty_set(self):
        s = '{"q":"hello"}'
        assert _strip_volatile_fields(s, set()) == s

    def test_non_dict_returns_original(self):
        assert _strip_volatile_fields('"str"', set()) == '"str"'

    def test_invalid_json_returns_original(self):
        assert _strip_volatile_fields('not-json', {'x'}) == 'not-json'


class TestAutoInferVolatile:
    def test_does_not_infer_only_changing_field(self):
        """If only one field exists and it changes, it's NOT volatile (no constant to anchor)."""
        last = {"q": "a"}
        inf = _auto_infer_volatile(last, '{"q":"b"}')
        assert 'q' not in inf, f'q should NOT be inferred: {inf}'

    def test_infers_repeating_field_with_constant(self):
        last = {"q": "x", "rid": 1}
        inf = _auto_infer_volatile(last, '{"q":"x","rid":2}')
        assert 'rid' in inf, f'rid should be inferred: {inf}'

    def test_no_inference_on_non_dict_current(self):
        inf = _auto_infer_volatile({"x": 1}, '"string"')
        assert len(inf) == 0

    def test_no_inference_when_last_is_none(self):
        inf = _auto_infer_volatile(None, '{"q":"a"}')
        assert len(inf) == 0

    def test_multiple_diffs_no_inference(self):
        last = {"q": "x", "rid": 1, "ts": "a"}
        inf = _auto_infer_volatile(last, '{"q":"y","rid":2,"ts":"a"}')
        assert len(inf) == 0  # 2 diffs, not exactly 1

    def test_no_inference_no_constants(self):
        last = {"q": "x"}
        inf = _auto_infer_volatile(last, '{"q":"y"}')
        assert len(inf) == 0  # no constant fields to anchor


# ==========================================================================
# Unit: Hash-based trajectory tracking (CWE-400 mitigation)
# ==========================================================================

class TestComputeTrajectoryHash:
    def test_deterministic(self):
        h1 = _compute_trajectory_hash("search", '{"q":"x"}')
        h2 = _compute_trajectory_hash("search", '{"q":"x"}')
        assert h1 == h2

    def test_different_tool_different_hash(self):
        h1 = _compute_trajectory_hash("search", '{"q":"x"}')
        h2 = _compute_trajectory_hash("read", '{"q":"x"}')
        assert h1 != h2

    def test_different_args_different_hash(self):
        h1 = _compute_trajectory_hash("search", '{"q":"x"}')
        h2 = _compute_trajectory_hash("search", '{"q":"y"}')
        assert h1 != h2

    def test_fixed_size(self):
        h = _compute_trajectory_hash("x" * 10000, "y" * 100000)
        assert len(h) == 64  # SHA-256 hex is always 64 chars


class TestCallHistory:
    def test_stores_only_hash_no_raw_json(self):
        h = _CallHistory()
        h.append("s1", "search", "a" * 64)
        entries = h._store["s1"]
        assert len(entries) == 1
        tool, hash_val = entries[0]
        assert tool == "search"
        assert len(hash_val) == 64

    def test_get_recent_respects_window(self):
        h = _CallHistory()
        for i in range(10):
            h.append("s1", "t", f"{i:064d}")
        recent = h.get_recent("s1", 3)
        assert len(recent) == 3
        # session has 10 entries (0-9), window=3 → entries 7, 8, 9
        assert recent[0][1] == f"{7:064d}"
        assert recent[1][1] == f"{8:064d}"
        assert recent[2][1] == f"{9:064d}"

    def test_memory_cap_eviction(self):
        h = _CallHistory()
        h.MAX_SESSIONS = 3
        for i in range(5):
            h.append(f"s{i}", "t", f"hash_{i:064d}")
        assert len(h._store) <= 3

    def test_clear_removes_session(self):
        h = _CallHistory()
        h.append("s1", "t", "hash_0000000000000000000000000000000000000000000000000000000000000000")
        assert "s1" in h._store
        h.clear("s1")
        assert "s1" not in h._store

    def test_trim(self):
        h = _CallHistory()
        for i in range(10):
            h.append("s1", "t", f"hash_{i:064d}")
        h.trim("s1", 3)
        assert len(h._store["s1"]) == 3


# ==========================================================================
# Integration: MicroloopGuardrail
# ==========================================================================

class TestMicroloopGuardrail:

    def test_allows_unique_calls(self):
        g = MicroloopGuardrail(max_repeats=3)
        g._check_and_record('s1', 'search', '{"q":"a"}')
        g._check_and_record('s1', 'search', '{"q":"b"}')
        g._check_and_record('s1', 'search', '{"q":"c"}')

    def test_blocks_identical_calls(self):
        g = MicroloopGuardrail(max_repeats=3)
        g._check_and_record('s1', 'search', '{"q":"x"}')
        g._check_and_record('s1', 'search', '{"q":"x"}')
        with pytest.raises(MicroloopLoopDetected) as exc:
            g._check_and_record('s1', 'search', '{"q":"x"}')
        info = exc.value
        assert info.tool_name == 'search'
        assert info.repeat_count == 3
        assert '3x' in str(info)

    def test_blocks_with_configured_volatile_fields(self):
        g = MicroloopGuardrail(max_repeats=3, volatile_fields=['req_id'])
        g._check_and_record('s1', 'search', '{"q":"x","req_id":1}')
        g._check_and_record('s1', 'search', '{"q":"x","req_id":2}')
        with pytest.raises(MicroloopLoopDetected):
            g._check_and_record('s1', 'search', '{"q":"x","req_id":3}')

    def test_auto_infers_volatile_fields(self):
        g = MicroloopGuardrail(max_repeats=3, auto_infer_volatile=True)
        g._check_and_record('s1', 'search', '{"q":"x","rid":1}')
        g._check_and_record('s1', 'search', '{"q":"x","rid":2}')
        with pytest.raises(MicroloopLoopDetected):
            g._check_and_record('s1', 'search', '{"q":"x","rid":3}')

    def test_no_false_positive_when_query_changes(self):
        g = MicroloopGuardrail(max_repeats=3)
        g._check_and_record('s1', 'search', '{"q":"a","rid":1}')
        g._check_and_record('s1', 'search', '{"q":"b","rid":2}')
        g._check_and_record('s1', 'search', '{"q":"c","rid":3}')

    def test_session_isolation(self):
        g = MicroloopGuardrail(max_repeats=2)
        g._check_and_record('s1', 'search', '{"q":"x"}')
        g._check_and_record('s2', 'search', '{"q":"x"}')

    def test_respects_history_window(self):
        g = MicroloopGuardrail(max_repeats=3, history_window=2)
        g._check_and_record('s1', 'read', '{}')
        g._check_and_record('s1', 'search', '{"q":"x"}')
        g._check_and_record('s1', 'read', '{}')
        # window=2, only 1 identical 'search' in window
        g._check_and_record('s1', 'search', '{"q":"x"}')
        g._check_and_record('s1', 'search', '{"q":"x"}')  # 2 in window
        with pytest.raises(MicroloopLoopDetected):
            g._check_and_record('s1', 'search', '{"q":"x"}')  # 3 → block

    def test_extract_tool_calls_openai(self):
        messages = [
            {'role': 'user', 'content': 'hi'},
            {'role': 'assistant', 'tool_calls': [
                {'id': 'c1', 'function': {'name': 'search', 'arguments': '{"q":"x"}'}}]}
        ]
        pairs = MicroloopGuardrail._extract_tool_calls(messages)
        assert len(pairs) == 1
        assert pairs[0] == ('search', '{"q":"x"}')

    def test_extract_tool_calls_empty_and_non_list(self):
        assert MicroloopGuardrail._extract_tool_calls([]) == []
        assert MicroloopGuardrail._extract_tool_calls('not_a_list') == []

    def test_hook_allows_different_queries(self):
        g = MicroloopGuardrail(max_repeats=3)
        data = {
            'messages': [
                {'role': 'user', 'content': 'find X then find Y'},
                {'role': 'assistant', 'tool_calls': [
                    {'id': '1', 'function': {'name': 'search', 'arguments': '{"q":"X"}'}}]},
                {'role': 'tool', 'tool_call_id': '1', 'content': 'X results'},
                {'role': 'assistant', 'tool_calls': [
                    {'id': '2', 'function': {'name': 'search', 'arguments': '{"q":"Y"}'}}]},
            ]
        }
        asyncio.run(g.async_pre_call_hook(data=data))
        # Should complete without raising MicroloopLoopDetected

    def test_hook_blocks_repeated_loop(self):
        g = MicroloopGuardrail(max_repeats=3)
        data = {
            'messages': [
                {'role': 'user', 'content': 'keep searching'},
                {'role': 'assistant', 'tool_calls': [
                    {'id': '1', 'function': {'name': 'search', 'arguments': '{"q":"X"}'}}]},
                {'role': 'tool', 'tool_call_id': '1', 'content': 'ok'},
                {'role': 'assistant', 'tool_calls': [
                    {'id': '2', 'function': {'name': 'search', 'arguments': '{"q":"X"}'}}]},
            ]
        }
        # Each hook call adds 1 to history. With max_repeats=3, need 3 calls to block.
        asyncio.run(g.async_pre_call_hook(data=data))
        asyncio.run(g.async_pre_call_hook(data=data))

        # Third identical call: history has 2 prior → 3 total, block
        with pytest.raises(MicroloopLoopDetected):
            asyncio.run(g.async_pre_call_hook(data=data))


