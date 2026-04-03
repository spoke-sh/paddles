use super::context_quality::ContextStrain;
use super::harness::{
    GovernorState, HarnessChamber, HarnessSnapshot, HarnessStatus, TimeoutPhase, TimeoutState,
};
use super::interpretation::InterpretationContext;
use super::traces::{TraceModelExchangeCategory, TraceModelExchangeLane, TraceModelExchangePhase};
use paddles_conversation::TraceArtifactId;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnIntent {
    Casual,
    DirectResponse,
    DeterministicAction,
    Planned,
}

impl TurnIntent {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Casual => "casual",
            Self::DirectResponse => "direct-response",
            Self::DeterministicAction => "deterministic-action",
            Self::Planned => "planned",
        }
    }

    pub fn uses_planner(&self) -> bool {
        matches!(self, Self::Planned)
    }

    pub fn prefers_tools(&self) -> bool {
        matches!(self, Self::DeterministicAction)
    }

    pub fn is_casual(&self) -> bool {
        matches!(self, Self::Casual)
    }
}

impl fmt::Display for TurnIntent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TurnEvent {
    IntentClassified {
        intent: TurnIntent,
    },
    InterpretationContext {
        context: InterpretationContext,
    },
    GuidanceGraphExpanded {
        depth: usize,
        document_count: usize,
        sources: Vec<String>,
    },
    RouteSelected {
        summary: String,
    },
    PlannerCapability {
        provider: String,
        capability: String,
    },
    GathererCapability {
        provider: String,
        capability: String,
    },
    PlannerActionSelected {
        sequence: usize,
        action: String,
        rationale: String,
    },
    ThreadCandidateCaptured {
        candidate_id: String,
        active_thread: String,
        prompt: String,
    },
    ThreadDecisionApplied {
        candidate_id: String,
        decision: String,
        target_thread: String,
        rationale: String,
    },
    ThreadMerged {
        source_thread: String,
        target_thread: String,
        mode: String,
        summary: Option<String>,
    },
    PlannerStepProgress {
        step_number: usize,
        step_limit: usize,
        action: String,
        query: Option<String>,
        evidence_count: usize,
    },
    GathererSearchProgress {
        phase: String,
        elapsed_seconds: u64,
        eta_seconds: Option<u64>,
        strategy: Option<String>,
        detail: Option<String>,
    },
    GathererSummary {
        provider: String,
        summary: String,
        sources: Vec<String>,
    },
    HarnessState {
        snapshot: HarnessSnapshot,
    },
    PlannerSummary {
        strategy: String,
        mode: String,
        turns: usize,
        steps: usize,
        stop_reason: Option<String>,
        active_branch_id: Option<String>,
        branch_count: Option<usize>,
        frontier_count: Option<usize>,
        node_count: Option<usize>,
        edge_count: Option<usize>,
        retained_artifact_count: Option<usize>,
    },
    RefinementApplied {
        reason: String,
        before_summary: String,
        after_summary: String,
    },
    ContextAssembly {
        label: String,
        hits: usize,
        retained_artifacts: usize,
        pruned_artifacts: usize,
    },
    ToolCalled {
        call_id: String,
        tool_name: String,
        invocation: String,
    },
    ToolFinished {
        call_id: String,
        tool_name: String,
        summary: String,
    },
    Fallback {
        stage: String,
        reason: String,
    },
    #[serde(rename = "context_strain")]
    ContextStrain {
        strain: ContextStrain,
    },
    SynthesisReady {
        grounded: bool,
        citations: Vec<String>,
        insufficient_evidence: bool,
    },
}

impl TurnEvent {
    pub fn event_type_key(&self) -> &'static str {
        match self {
            Self::IntentClassified { .. } => "intent_classified",
            Self::InterpretationContext { .. } => "interpretation_context",
            Self::GuidanceGraphExpanded { .. } => "guidance_graph_expanded",
            Self::RouteSelected { .. } => "route_selected",
            Self::PlannerCapability { .. } => "planner_capability",
            Self::GathererCapability { .. } => "gatherer_capability",
            Self::PlannerActionSelected { .. } => "planner_action_selected",
            Self::ThreadCandidateCaptured { .. } => "thread_candidate_captured",
            Self::ThreadDecisionApplied { .. } => "thread_decision_applied",
            Self::ThreadMerged { .. } => "thread_merged",
            Self::PlannerStepProgress { .. } => "planner_step_progress",
            Self::GathererSearchProgress { .. } => "gatherer_search_progress",
            Self::GathererSummary { .. } => "gatherer_summary",
            Self::HarnessState { .. } => "harness_state",
            Self::PlannerSummary { .. } => "planner_summary",
            Self::RefinementApplied { .. } => "refinement_applied",
            Self::ContextAssembly { .. } => "context_assembly",
            Self::ToolCalled { .. } => "tool_called",
            Self::ToolFinished { .. } => "tool_finished",
            Self::Fallback { .. } => "fallback",
            Self::ContextStrain { .. } => "context_strain",
            Self::SynthesisReady { .. } => "synthesis_ready",
        }
    }

    /// Inherent minimum verbosity tier for this event kind.
    /// Pace classification can promote events below their inherent level.
    pub fn min_verbosity(&self) -> u8 {
        match self {
            Self::PlannerStepProgress { .. }
            | Self::GathererSearchProgress { .. }
            | Self::ToolCalled { .. }
            | Self::ToolFinished { .. }
            | Self::Fallback { .. }
            | Self::SynthesisReady { .. } => 0,

            Self::PlannerActionSelected { .. }
            | Self::GathererSummary { .. }
            | Self::HarnessState { .. }
            | Self::PlannerSummary { .. }
            | Self::ContextAssembly { .. }
            | Self::ContextStrain { .. }
            | Self::RefinementApplied { .. }
            | Self::ThreadDecisionApplied { .. }
            | Self::GuidanceGraphExpanded { .. }
            | Self::ThreadMerged { .. } => 1,

            Self::IntentClassified { .. }
            | Self::InterpretationContext { .. }
            | Self::RouteSelected { .. }
            | Self::PlannerCapability { .. }
            | Self::GathererCapability { .. }
            | Self::ThreadCandidateCaptured { .. } => 2,
        }
    }

    pub fn derived_harness_snapshot(&self) -> Option<HarnessSnapshot> {
        match self {
            Self::HarnessState { .. } => None,
            Self::IntentClassified { intent } => Some(
                HarnessSnapshot::active(HarnessChamber::Routing)
                    .with_detail(format!("intent={}", intent.label())),
            ),
            Self::InterpretationContext { context } => Some(
                HarnessSnapshot::active(HarnessChamber::Interpretation)
                    .with_detail(context.summary.clone()),
            ),
            Self::GuidanceGraphExpanded {
                depth,
                document_count,
                ..
            } => Some(
                HarnessSnapshot::active(HarnessChamber::Interpretation).with_detail(format!(
                    "guidance graph depth {depth}, {document_count} docs"
                )),
            ),
            Self::RouteSelected { summary } => {
                Some(HarnessSnapshot::active(HarnessChamber::Routing).with_detail(summary.clone()))
            }
            Self::PlannerCapability {
                provider,
                capability,
            } => Some(
                HarnessSnapshot::active(HarnessChamber::Planning)
                    .with_detail(format!("{provider}: {capability}")),
            ),
            Self::GathererCapability {
                provider,
                capability,
            } => Some(
                HarnessSnapshot::active(HarnessChamber::Gathering)
                    .with_detail(format!("{provider}: {capability}")),
            ),
            Self::PlannerActionSelected {
                sequence, action, ..
            } => Some(
                HarnessSnapshot::active(HarnessChamber::Planning)
                    .with_detail(format!("step {sequence}: {action}")),
            ),
            Self::ThreadCandidateCaptured {
                candidate_id,
                active_thread,
                ..
            } => Some(
                HarnessSnapshot::active(HarnessChamber::Threading)
                    .with_detail(format!("{candidate_id} on {active_thread}")),
            ),
            Self::ThreadDecisionApplied {
                decision,
                target_thread,
                ..
            } => Some(
                HarnessSnapshot::active(HarnessChamber::Threading)
                    .with_detail(format!("{decision} -> {target_thread}")),
            ),
            Self::ThreadMerged {
                source_thread,
                target_thread,
                mode,
                ..
            } => Some(
                HarnessSnapshot::active(HarnessChamber::Threading)
                    .with_detail(format!("{source_thread} -> {target_thread} via {mode}")),
            ),
            Self::PlannerStepProgress {
                step_number,
                step_limit,
                action,
                query,
                ..
            } => {
                let query = query
                    .as_deref()
                    .map(|value| format!(" — {value}"))
                    .unwrap_or_default();
                Some(
                    HarnessSnapshot::active(HarnessChamber::Planning)
                        .with_detail(format!("step {step_number}/{step_limit}: {action}{query}")),
                )
            }
            Self::GathererSearchProgress {
                phase,
                elapsed_seconds,
                eta_seconds,
                strategy,
                detail,
            } => {
                let mut governor = GovernorState::active()
                    .with_timeout(timeout_state(*elapsed_seconds, *eta_seconds));
                if matches!(
                    governor.timeout.phase,
                    TimeoutPhase::Stalled | TimeoutPhase::Expired
                ) {
                    governor.status = HarnessStatus::Intervening;
                    governor.intervention = Some(format!(
                        "search {phase} is {}",
                        governor.timeout.phase.label()
                    ));
                }
                let detail = detail
                    .clone()
                    .or_else(|| {
                        strategy
                            .as_ref()
                            .map(|value| format!("{phase} via {value}"))
                    })
                    .unwrap_or_else(|| phase.clone());
                Some(
                    HarnessSnapshot::active(HarnessChamber::Gathering)
                        .with_governor(governor)
                        .with_detail(detail),
                )
            }
            Self::GathererSummary {
                provider, summary, ..
            } => Some(
                HarnessSnapshot::active(HarnessChamber::Gathering)
                    .with_detail(format!("{provider}: {summary}")),
            ),
            Self::PlannerSummary {
                strategy,
                mode,
                steps,
                stop_reason,
                ..
            } => {
                let mut detail = format!("{strategy} {mode} ({steps} steps)");
                if let Some(stop_reason) = stop_reason {
                    detail.push_str(&format!(" · stop={stop_reason}"));
                }
                Some(HarnessSnapshot::active(HarnessChamber::Planning).with_detail(detail))
            }
            Self::RefinementApplied { reason, .. } => Some(
                HarnessSnapshot::intervening(HarnessChamber::Governor, reason.clone())
                    .with_detail(reason.clone()),
            ),
            Self::ContextAssembly {
                label,
                hits,
                retained_artifacts,
                ..
            } => Some(
                HarnessSnapshot::active(HarnessChamber::Interpretation).with_detail(format!(
                    "{label}: {hits} hits, retained {retained_artifacts}"
                )),
            ),
            Self::ToolCalled {
                tool_name,
                invocation,
                ..
            } => Some(
                HarnessSnapshot::active(HarnessChamber::Tooling)
                    .with_detail(format!("{tool_name}: {invocation}")),
            ),
            Self::ToolFinished {
                tool_name, summary, ..
            } => Some(
                HarnessSnapshot::active(HarnessChamber::Tooling)
                    .with_detail(format!("{tool_name}: {summary}")),
            ),
            Self::Fallback { stage, reason } => Some(
                HarnessSnapshot::intervening(
                    HarnessChamber::Governor,
                    format!("{stage}: {reason}"),
                )
                .with_detail(format!("{stage}: {reason}")),
            ),
            Self::ContextStrain { strain } => Some(
                HarnessSnapshot::intervening(
                    HarnessChamber::Governor,
                    format!("context strain {}", strain.level.label()),
                )
                .with_detail(format!(
                    "{} truncation(s), factors: {}",
                    strain.truncation_count,
                    strain
                        .factors
                        .iter()
                        .map(|factor| factor.label())
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            ),
            Self::SynthesisReady {
                grounded,
                citations,
                insufficient_evidence,
            } => {
                let detail = if *insufficient_evidence {
                    "insufficient evidence".to_string()
                } else if *grounded {
                    format!("grounded answer with {} citation(s)", citations.len())
                } else {
                    "direct answer ready".to_string()
                };
                Some(HarnessSnapshot::active(HarnessChamber::Rendering).with_detail(detail))
            }
        }
    }
}

fn timeout_state(elapsed_seconds: u64, eta_seconds: Option<u64>) -> TimeoutState {
    let deadline_seconds = eta_seconds.map(|eta| elapsed_seconds.saturating_add(eta));
    let phase = if elapsed_seconds >= 120 {
        TimeoutPhase::Expired
    } else if elapsed_seconds >= 45 {
        TimeoutPhase::Stalled
    } else if elapsed_seconds >= 10 {
        TimeoutPhase::Slow
    } else {
        TimeoutPhase::Nominal
    };
    TimeoutState {
        phase,
        elapsed_seconds: Some(elapsed_seconds),
        deadline_seconds,
    }
}

pub trait TurnEventSink: Send + Sync {
    fn emit(&self, event: TurnEvent);

    fn forensic_trace_sink(&self) -> Option<&dyn ForensicTraceSink> {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForensicArtifactCapture {
    pub exchange_id: String,
    pub lane: TraceModelExchangeLane,
    pub category: TraceModelExchangeCategory,
    pub phase: TraceModelExchangePhase,
    pub provider: String,
    pub model: String,
    pub parent_artifact_id: Option<TraceArtifactId>,
    pub summary: String,
    pub content: String,
    pub mime_type: String,
    pub labels: BTreeMap<String, String>,
}

pub trait ForensicTraceSink: Send + Sync {
    fn allocate_model_exchange_id(
        &self,
        lane: TraceModelExchangeLane,
        category: TraceModelExchangeCategory,
    ) -> String;

    fn record_forensic_artifact(&self, capture: ForensicArtifactCapture)
    -> Option<TraceArtifactId>;
}

#[derive(Default)]
pub struct NullTurnEventSink;

impl TurnEventSink for NullTurnEventSink {
    fn emit(&self, _event: TurnEvent) {}
}

/// Forwards events to multiple sinks.
pub struct MultiplexEventSink {
    sinks: Vec<Arc<dyn TurnEventSink>>,
}

impl MultiplexEventSink {
    pub fn new(sinks: Vec<Arc<dyn TurnEventSink>>) -> Self {
        Self { sinks }
    }
}

impl TurnEventSink for MultiplexEventSink {
    fn emit(&self, event: TurnEvent) {
        for sink in &self.sinks {
            sink.emit(event.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TurnEvent;
    use crate::domain::model::{
        ContextStrain, GovernorState, HarnessChamber, HarnessSnapshot, HarnessStatus, StrainFactor,
        TimeoutPhase, TimeoutState,
    };

    #[test]
    fn context_pressure_event_uses_context_strain_key() {
        let event = TurnEvent::ContextStrain {
            strain: ContextStrain::new(vec![StrainFactor::MemoryTruncated], 1),
        };

        assert_eq!(event.event_type_key(), "context_strain");
    }

    #[test]
    fn harness_state_event_uses_harness_state_key() {
        let event = TurnEvent::HarnessState {
            snapshot: HarnessSnapshot {
                chamber: HarnessChamber::Planning,
                governor: GovernorState {
                    status: HarnessStatus::Active,
                    timeout: TimeoutState {
                        phase: TimeoutPhase::Nominal,
                        elapsed_seconds: Some(2),
                        deadline_seconds: Some(30),
                    },
                    intervention: None,
                },
                detail: Some("planner step 1".to_string()),
            },
        };

        assert_eq!(event.event_type_key(), "harness_state");
    }
}
