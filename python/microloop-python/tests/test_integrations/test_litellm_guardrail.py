"""Tests for the Microloop LiteLLM Guardrail integration.

Mocks the Rust-backed ``microloop.microloop_core`` engine so these
tests execute in CI where the native binary is not available.
"""

from __future__ import annotations

import os
import sys
from unittest.mock import MagicMock, patch

import pytest

# Ensure the microloop package is importable from the repo root
_pkg_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
if _pkg_root not in sys.path:
    sys.path.insert(0, _pkg_root)

# Mock the Rust engine for CI coverage
_mock_core = MagicMock()
_mock_engine = MagicMock()
_mock_engine.verify.return_value = 0  # Default: Allow
_mock_core.Microloop = MagicMock(return_value=_mock_engine)
sys.modules["microloop.microloop_core"] = _mock_core


# Mock LiteLLM for environments where it is not installed
# CustomGuardrail must be a real class so the guardrail can inherit.
class _FakeCustomGuardrail:
    def __init__(
        self, guardrail_name="", supported_event_hooks=None, default_on=True, **kwargs
    ):
        self.guardrail_name = guardrail_name
        self.supported_event_hooks = supported_event_hooks or []
        self.default_on = default_on


class _FakeBadRequestError(Exception):
    pass


_mock_cg_mod = MagicMock()
_mock_cg_mod.CustomGuardrail = _FakeCustomGuardrail
sys.modules["litellm.integrations.custom_guardrail"] = _mock_cg_mod

_mock_exc_mod = MagicMock()
_mock_exc_mod.BadRequestError = _FakeBadRequestError
sys.modules["litellm.exceptions"] = _mock_exc_mod

sys.modules["litellm"] = MagicMock()
sys.modules["litellm.integrations"] = MagicMock()

# Now safe to import - all deps are mocked
from microloop.integrations import MicroloopLiteLLMGuardrail
from microloop.integrations.litellm import MicroloopLoopDetected


# ---- Fixtures ----


@pytest.fixture
def guardrail():
    """Return a fresh guardrail instance with a clean engine store."""
    _mock_engine.verify.return_value = 0
    _mock_core.Microloop.reset_mock()
    return MicroloopLiteLLMGuardrail(max_repeats=3, history_window=10)


# ---- Allow (no loop detected) ----


@pytest.mark.asyncio
async def test_allow_when_no_tool_call(guardrail):
    """Request without tool calls should pass through unmodified."""
    result = await guardrail.async_pre_call_hook(
        {"messages": [{"role": "user", "content": "Hello"}]},
        {},
    )
    assert result is None


@pytest.mark.asyncio
async def test_allow_when_verify_returns_zero(guardrail):
    """Tool call that passes verification is allowed."""
    _mock_engine.verify.return_value = 0
    result = await guardrail.async_pre_call_hook(
        {
            "messages": [
                {
                    "role": "assistant",
                    "tool_calls": [
                        {
                            "function": {
                                "name": "search_web",
                                "arguments": '{"q": "hello"}',
                            }
                        }
                    ],
                }
            ],
            "metadata": {"session_id": "test-session-2"},
        },
        {},
    )
    assert result is None


# ---- Block (loop detected) ----


@pytest.mark.asyncio
async def test_block_when_verify_returns_one(guardrail):
    """Failing verification raises LoopDetected."""
    _mock_engine.verify.return_value = 1
    with pytest.raises(MicroloopLoopDetected) as ex:
        await guardrail.async_pre_call_hook(
            {
                "messages": [
                    {
                        "role": "assistant",
                        "tool_calls": [
                            {
                                "function": {
                                    "name": "search_web",
                                    "arguments": '{"q": "looping"}',
                                }
                            }
                        ],
                    }
                ],
                "metadata": {"session_id": "test-session-3"},
            },
            {},
        )
    assert ex.value.tool_name == "search_web"
    assert ex.value.max_repeats == 3


@pytest.mark.asyncio
async def test_block_reports_correct_tool_name(guardrail):
    """Exception contains the offending tool name."""
    _mock_engine.verify.return_value = 1
    with pytest.raises(MicroloopLoopDetected) as ex:
        await guardrail.async_pre_call_hook(
            {
                "messages": [
                    {
                        "role": "assistant",
                        "tool_calls": [
                            {
                                "function": {
                                    "name": "generate_code",
                                    "arguments": '{"task": "fix"}',
                                }
                            }
                        ],
                    }
                ],
                "metadata": {"session_id": "test-session-4"},
            },
            {},
        )
    assert ex.value.tool_name == "generate_code"


# ---- Session ID extraction ----


def test_session_id_from_metadata():
    """Extract session_id from metadata."""
    s = MicroloopLiteLLMGuardrail._get_session_id(
        {"metadata": {"session_id": "my-session"}}
    )
    assert s == "my-session"


def test_session_id_from_litellm_session_id():
    """Fall back to litellm_session_id."""
    s = MicroloopLiteLLMGuardrail._get_session_id(
        {"metadata": {}, "litellm_session_id": "proxy"}
    )
    assert s == "proxy"


def test_session_id_fallback_is_per_request_uuid():
    """Fallback generates a unique per-request UUID."""
    s1 = MicroloopLiteLLMGuardrail._get_session_id({"metadata": {}})
    s2 = MicroloopLiteLLMGuardrail._get_session_id({"metadata": {}})
    assert s1.startswith("req_")
    assert s2.startswith("req_")
    assert s1 != s2


# ---- Engine lifecycle ----


def test_engine_creation(guardrail):
    """New engine for unseen session ID."""
    _mock_engine.verify.reset_mock()
    e = guardrail._get_engine("brand-new-session")
    assert e is not None
    _mock_core.Microloop.assert_called()


# ---- Exception class ----


def test_loop_detected_exception_message():
    """Descriptive exception message."""
    ex = MicroloopLoopDetected(
        tool_name="search_web", repeat_count=3, max_repeats=3, session_id="test"
    )
    m = str(ex)
    assert "search_web" in m
    assert "3" in m
    assert "test" in m


def test_loop_detected_exception_no_session():
    """Without session ID, message omits it."""
    ex = MicroloopLoopDetected(tool_name="search_web", repeat_count=3, max_repeats=3)
    assert "search_web" in str(ex)
    assert "in session" not in str(ex)
