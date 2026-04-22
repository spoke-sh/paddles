use crate::infrastructure::providers::DeliberationState;
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeliberationSignals {
    pub continuation: DeliberationSignal<DeliberationContinuation>,
    pub uncertainty: DeliberationSignal<DeliberationConfidence>,
    pub evidence_gaps: DeliberationSignal<Vec<String>>,
    pub branch_candidates: DeliberationSignal<Vec<String>>,
    pub stop_confidence: DeliberationSignal<DeliberationConfidence>,
    pub risk_hints: DeliberationSignal<Vec<String>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum DeliberationSignal<T> {
    #[default]
    None,
    Unknown,
    Present(T),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliberationConfidence {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeliberationContinuation {
    pub reusable_state: bool,
    pub tool_results_required: bool,
}

impl DeliberationContinuation {
    const fn tool_results_required() -> Self {
        Self {
            reusable_state: true,
            tool_results_required: true,
        }
    }
}

pub fn extract_deliberation_signals(
    deliberation_state: Option<&DeliberationState>,
) -> DeliberationSignals {
    let Some(deliberation_state) = deliberation_state else {
        return DeliberationSignals::default();
    };

    let mut signals = DeliberationSignals {
        continuation: extract_continuation_signal(deliberation_state),
        ..DeliberationSignals::default()
    };
    if has_opaque_reasoning_text(deliberation_state) {
        signals.uncertainty = DeliberationSignal::Unknown;
        signals.evidence_gaps = DeliberationSignal::Unknown;
        signals.branch_candidates = DeliberationSignal::Unknown;
        signals.stop_confidence = DeliberationSignal::Unknown;
        signals.risk_hints = DeliberationSignal::Unknown;
    }
    signals
}

fn extract_continuation_signal(
    deliberation_state: &DeliberationState,
) -> DeliberationSignal<DeliberationContinuation> {
    match deliberation_state
        .payload()
        .get("kind")
        .and_then(Value::as_str)
    {
        Some("openai_responses")
            if deliberation_state
                .payload()
                .get("previous_response_id")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty()) =>
        {
            DeliberationSignal::Present(DeliberationContinuation::tool_results_required())
        }
        Some("moonshot_openai_chat_completion")
            if deliberation_state
                .payload()
                .get("assistant")
                .and_then(|assistant| assistant.get("tool_calls"))
                .and_then(Value::as_array)
                .is_some_and(|tool_calls| !tool_calls.is_empty()) =>
        {
            DeliberationSignal::Present(DeliberationContinuation::tool_results_required())
        }
        Some("anthropic_messages")
            if deliberation_state
                .payload()
                .get("assistant")
                .and_then(|assistant| assistant.get("content"))
                .and_then(Value::as_array)
                .is_some_and(|blocks| {
                    blocks
                        .iter()
                        .any(|block| block.get("type").and_then(Value::as_str) == Some("tool_use"))
                }) =>
        {
            DeliberationSignal::Present(DeliberationContinuation::tool_results_required())
        }
        Some("gemini_generate_content")
            if deliberation_state
                .payload()
                .get("model")
                .and_then(|model| model.get("parts"))
                .and_then(Value::as_array)
                .is_some_and(|parts| {
                    parts.iter().any(|part| {
                        part.get("functionCall")
                            .and_then(Value::as_object)
                            .is_some()
                    })
                }) =>
        {
            DeliberationSignal::Present(DeliberationContinuation::tool_results_required())
        }
        _ => DeliberationSignal::None,
    }
}

fn has_opaque_reasoning_text(deliberation_state: &DeliberationState) -> bool {
    match deliberation_state
        .payload()
        .get("kind")
        .and_then(Value::as_str)
    {
        Some("moonshot_openai_chat_completion") => deliberation_state
            .payload()
            .get("assistant")
            .and_then(|assistant| assistant.get("reasoning_content"))
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty()),
        Some("anthropic_messages") => deliberation_state
            .payload()
            .get("assistant")
            .and_then(|assistant| assistant.get("content"))
            .and_then(Value::as_array)
            .is_some_and(|blocks| {
                blocks.iter().any(|block| {
                    matches!(
                        block.get("type").and_then(Value::as_str),
                        Some("thinking") | Some("redacted_thinking")
                    )
                })
            }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DeliberationConfidence, DeliberationContinuation, DeliberationSignal, DeliberationSignals,
        extract_deliberation_signals,
    };
    use crate::infrastructure::providers::{DeliberationState, ModelProvider};
    use serde_json::json;

    #[test]
    fn extraction_defaults_to_explicit_none_without_provider_state() {
        assert_eq!(
            extract_deliberation_signals(None),
            DeliberationSignals::default()
        );
    }

    #[test]
    fn openai_responses_state_extracts_only_continuation_hints() {
        let state = DeliberationState::new(
            ModelProvider::Openai,
            "gpt-5.4-pro",
            json!({
                "kind": "openai_responses",
                "previous_response_id": "resp_1",
            }),
        );

        assert_eq!(
            extract_deliberation_signals(Some(&state)),
            DeliberationSignals {
                continuation: DeliberationSignal::Present(
                    DeliberationContinuation::tool_results_required(),
                ),
                uncertainty: DeliberationSignal::None,
                evidence_gaps: DeliberationSignal::None,
                branch_candidates: DeliberationSignal::None,
                stop_confidence: DeliberationSignal::None,
                risk_hints: DeliberationSignal::None,
            }
        );
    }

    #[test]
    fn moonshot_opaque_reasoning_maps_secondary_hints_to_unknown() {
        let state = DeliberationState::new(
            ModelProvider::Moonshot,
            "kimi-k2.6",
            json!({
                "kind": "moonshot_openai_chat_completion",
                "assistant": {
                    "role": "assistant",
                    "reasoning_content": "inspect trace recorder fallback state",
                    "tool_calls": [
                        {
                            "id": "call_1",
                            "type": "function",
                            "function": {
                                "name": "search",
                                "arguments": "{\"query\":\"trace recorder fallback\"}"
                            }
                        }
                    ]
                }
            }),
        );

        assert_eq!(
            extract_deliberation_signals(Some(&state)),
            DeliberationSignals {
                continuation: DeliberationSignal::Present(
                    DeliberationContinuation::tool_results_required(),
                ),
                uncertainty: DeliberationSignal::Unknown,
                evidence_gaps: DeliberationSignal::Unknown,
                branch_candidates: DeliberationSignal::Unknown,
                stop_confidence: DeliberationSignal::Unknown,
                risk_hints: DeliberationSignal::Unknown,
            }
        );
    }

    #[test]
    fn gemini_state_extracts_continuation_without_inventing_other_hints() {
        let state = DeliberationState::new(
            ModelProvider::Google,
            "gemini-2.5-flash",
            json!({
                "kind": "gemini_generate_content",
                "model": {
                    "role": "model",
                    "parts": [
                        {
                            "thoughtSignature": "sig_1",
                            "functionCall": {
                                "name": "search",
                                "args": {
                                    "query": "trace recorder fallback"
                                }
                            }
                        }
                    ]
                }
            }),
        );

        assert_eq!(
            extract_deliberation_signals(Some(&state)),
            DeliberationSignals {
                continuation: DeliberationSignal::Present(
                    DeliberationContinuation::tool_results_required(),
                ),
                uncertainty: DeliberationSignal::None,
                evidence_gaps: DeliberationSignal::None,
                branch_candidates: DeliberationSignal::None,
                stop_confidence: DeliberationSignal::None,
                risk_hints: DeliberationSignal::None,
            }
        );
    }

    #[test]
    fn anthropic_thinking_blocks_keep_secondary_hints_unknown_until_normalized() {
        let state = DeliberationState::new(
            ModelProvider::Anthropic,
            "claude-sonnet-4-20250514",
            json!({
                "kind": "anthropic_messages",
                "assistant": {
                    "role": "assistant",
                    "content": [
                        {
                            "type": "thinking",
                            "thinking": "inspect the trace recorder state before searching",
                            "signature": "sig_thinking"
                        },
                        {
                            "type": "tool_use",
                            "id": "toolu_test",
                            "name": "search",
                            "input": {
                                "query": "trace recorder fallback"
                            }
                        }
                    ]
                }
            }),
        );

        let signals = extract_deliberation_signals(Some(&state));

        assert_eq!(
            signals.continuation,
            DeliberationSignal::Present(DeliberationContinuation::tool_results_required())
        );
        assert_eq!(signals.uncertainty, DeliberationSignal::Unknown);
        assert_eq!(signals.evidence_gaps, DeliberationSignal::Unknown);
        assert_eq!(signals.branch_candidates, DeliberationSignal::Unknown);
        assert_eq!(signals.stop_confidence, DeliberationSignal::Unknown);
        assert_eq!(signals.risk_hints, DeliberationSignal::Unknown);
    }

    #[test]
    fn signal_contract_carries_bounded_confidence_levels() {
        let signals = DeliberationSignals {
            uncertainty: DeliberationSignal::Present(DeliberationConfidence::Medium),
            stop_confidence: DeliberationSignal::Present(DeliberationConfidence::Low),
            ..DeliberationSignals::default()
        };

        assert_eq!(
            signals.uncertainty,
            DeliberationSignal::Present(DeliberationConfidence::Medium)
        );
        assert_eq!(
            signals.stop_confidence,
            DeliberationSignal::Present(DeliberationConfidence::Low)
        );
    }
}
