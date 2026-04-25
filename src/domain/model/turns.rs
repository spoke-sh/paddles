use super::context_quality::ContextStrain;
use super::control::ControlResult;
use super::execution_hand::{ExecutionGovernanceDecision, ExecutionGovernanceSnapshot};
use super::harness::HarnessSnapshot;
use super::interpretation::InterpretationContext;
use super::traces::{TraceModelExchangeCategory, TraceModelExchangeLane, TraceModelExchangePhase};
use super::{CollaborationModeResult, StructuredClarificationResult};
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
pub struct AppliedEdit {
    pub files: Vec<String>,
    pub diff: String,
    pub insertions: usize,
    pub deletions: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<AppliedEditEvidence>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppliedEditEvidenceKind {
    Formatter,
    Diagnostics,
}

impl AppliedEditEvidenceKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Formatter => "formatter",
            Self::Diagnostics => "diagnostics",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppliedEditEvidenceStatus {
    Passed,
    Warning,
    Unavailable,
}

impl AppliedEditEvidenceStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AppliedEditEvidence {
    pub kind: AppliedEditEvidenceKind,
    pub status: AppliedEditEvidenceStatus,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanChecklistItemStatus {
    Pending,
    Completed,
}

impl PlanChecklistItemStatus {
    pub fn marker(self) -> &'static str {
        match self {
            Self::Pending => "□",
            Self::Completed => "✓",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlanChecklistItem {
    pub id: String,
    pub label: String,
    pub status: PlanChecklistItemStatus,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        signal_summary: Option<String>,
    },
    PlanUpdated {
        items: Vec<PlanChecklistItem>,
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
    CollaborationModeChanged {
        result: CollaborationModeResult,
    },
    StructuredClarificationChanged {
        result: StructuredClarificationResult,
    },
    ControlStateChanged {
        result: ControlResult,
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
    ExecutionGovernanceProfileApplied {
        snapshot: ExecutionGovernanceSnapshot,
    },
    ExecutionGovernanceDecisionRecorded {
        decision: ExecutionGovernanceDecision,
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
    ToolOutput {
        call_id: String,
        tool_name: String,
        stream: String,
        output: String,
    },
    ToolFinished {
        call_id: String,
        tool_name: String,
        summary: String,
    },
    WorkspaceEditApplied {
        call_id: String,
        tool_name: String,
        edit: AppliedEdit,
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
            Self::PlanUpdated { .. } => "plan_updated",
            Self::ThreadCandidateCaptured { .. } => "thread_candidate_captured",
            Self::ThreadDecisionApplied { .. } => "thread_decision_applied",
            Self::ThreadMerged { .. } => "thread_merged",
            Self::CollaborationModeChanged { .. } => "collaboration_mode_changed",
            Self::StructuredClarificationChanged { .. } => "structured_clarification_changed",
            Self::ControlStateChanged { .. } => "control_state_changed",
            Self::PlannerStepProgress { .. } => "planner_step_progress",
            Self::GathererSearchProgress { .. } => "gatherer_search_progress",
            Self::GathererSummary { .. } => "gatherer_summary",
            Self::ExecutionGovernanceProfileApplied { .. } => {
                "execution_governance_profile_applied"
            }
            Self::ExecutionGovernanceDecisionRecorded { .. } => {
                "execution_governance_decision_recorded"
            }
            Self::HarnessState { .. } => "harness_state",
            Self::PlannerSummary { .. } => "planner_summary",
            Self::RefinementApplied { .. } => "refinement_applied",
            Self::ContextAssembly { .. } => "context_assembly",
            Self::ToolCalled { .. } => "tool_called",
            Self::ToolOutput { .. } => "tool_output",
            Self::ToolFinished { .. } => "tool_finished",
            Self::WorkspaceEditApplied { .. } => "workspace_edit_applied",
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
            | Self::PlanUpdated { .. }
            | Self::GathererSearchProgress { .. }
            | Self::ToolCalled { .. }
            | Self::ToolOutput { .. }
            | Self::ToolFinished { .. }
            | Self::WorkspaceEditApplied { .. }
            | Self::ControlStateChanged { .. }
            | Self::CollaborationModeChanged { .. }
            | Self::StructuredClarificationChanged { .. }
            | Self::ExecutionGovernanceProfileApplied { .. }
            | Self::ExecutionGovernanceDecisionRecorded { .. }
            | Self::Fallback { .. }
            | Self::SynthesisReady { grounded: true, .. }
            | Self::SynthesisReady {
                insufficient_evidence: true,
                ..
            } => 0,

            Self::SynthesisReady { .. } => 1,

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

    pub fn allows_pace_promotion(&self) -> bool {
        !matches!(
            self,
            Self::IntentClassified { .. }
                | Self::RouteSelected { .. }
                | Self::HarnessState { .. }
                | Self::SynthesisReady { .. }
        )
    }

    pub fn in_flight_label(&self) -> &'static str {
        match self {
            Self::HarnessState { snapshot } => match snapshot.chamber {
                super::harness::HarnessChamber::Idle => "Thinking",
                super::harness::HarnessChamber::Interpretation => "Interpreting",
                super::harness::HarnessChamber::Routing => "Routing",
                super::harness::HarnessChamber::Planning => "Planning",
                super::harness::HarnessChamber::Gathering => "Hunting",
                super::harness::HarnessChamber::Tooling => "Running tool",
                super::harness::HarnessChamber::Threading => "Threading",
                super::harness::HarnessChamber::Generation => "Generating response",
                super::harness::HarnessChamber::Rendering => "Rendering",
                super::harness::HarnessChamber::Governor => "Intervening",
            },
            Self::PlannerCapability { .. } => "Planning",
            Self::GathererCapability { .. } => "Gathering evidence",
            Self::CollaborationModeChanged { .. } => "Routing",
            Self::StructuredClarificationChanged { .. } => "Intervening",
            Self::ExecutionGovernanceProfileApplied { .. }
            | Self::ExecutionGovernanceDecisionRecorded { .. } => "Governing",
            Self::IntentClassified { .. } | Self::InterpretationContext { .. } => "Routing",
            Self::GuidanceGraphExpanded { .. } => "Interpreting",
            Self::RouteSelected { .. } => "Synthesizing",
            Self::PlannerActionSelected { .. }
            | Self::PlanUpdated { .. }
            | Self::PlannerStepProgress { .. } => "Planning",
            Self::PlannerSummary { .. } => "Synthesizing",
            Self::GathererSearchProgress { .. } | Self::GathererSummary { .. } => "Hunting",
            Self::ContextAssembly { .. } | Self::ContextStrain { .. } => "Thinking",
            Self::RefinementApplied { .. } => "Applying refinement",
            Self::ToolCalled { .. } | Self::ToolOutput { .. } => "Running tool",
            Self::ToolFinished { .. } | Self::WorkspaceEditApplied { .. } => "Thinking",
            Self::ThreadCandidateCaptured { .. }
            | Self::ThreadDecisionApplied { .. }
            | Self::ThreadMerged { .. } => "Threading",
            Self::ControlStateChanged { .. } => "Intervening",
            Self::Fallback { .. } => "Recovering",
            Self::SynthesisReady { .. } => "Rendering",
        }
    }

    pub fn is_search_progress(&self) -> bool {
        matches!(self, Self::GathererSearchProgress { .. })
    }

    pub fn is_planner_progress(&self) -> bool {
        matches!(self, Self::PlannerStepProgress { .. })
    }

    pub fn is_gathering_harness_progress(&self) -> bool {
        matches!(
            self,
            Self::HarnessState { snapshot }
                if snapshot.chamber == super::harness::HarnessChamber::Gathering
        )
    }

    /// Whether this event should be forwarded to long-lived projection streams.
    pub fn should_emit_to_projection_stream(&self) -> bool {
        match self {
            Self::HarnessState { snapshot } => snapshot.should_emit_to_stream(),
            _ => true,
        }
    }

    pub fn is_visible_at_verbosity(&self, verbose: u8) -> bool {
        self.should_emit_to_projection_stream() && self.min_verbosity() <= verbose
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
    use super::{TurnEvent, TurnIntent};
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

    #[test]
    fn harness_state_respects_projection_stream_policy() {
        let silent_snapshot = HarnessSnapshot::active(HarnessChamber::Gathering);
        let active_event = TurnEvent::HarnessState {
            snapshot: silent_snapshot,
        };

        let intervening_snapshot = HarnessSnapshot::intervening(HarnessChamber::Planning, "test");
        let intervening_event = TurnEvent::HarnessState {
            snapshot: intervening_snapshot,
        };

        assert!(!active_event.should_emit_to_projection_stream());
        assert!(intervening_event.should_emit_to_projection_stream());
    }

    #[test]
    fn non_harness_events_always_emit_to_projection_stream() {
        let event = TurnEvent::IntentClassified {
            intent: TurnIntent::Casual,
        };

        assert!(event.should_emit_to_projection_stream());
    }

    #[test]
    fn event_visibility_tracks_projection_stream_floor_and_verbosity() {
        let high_detail = TurnEvent::IntentClassified {
            intent: TurnIntent::Casual,
        };
        let low_detail = TurnEvent::ToolCalled {
            call_id: "call-1".to_string(),
            tool_name: "shell".to_string(),
            invocation: "pwd".to_string(),
        };

        assert!(!high_detail.is_visible_at_verbosity(0));
        assert!(!high_detail.is_visible_at_verbosity(1));
        assert!(high_detail.is_visible_at_verbosity(2));
        assert!(low_detail.is_visible_at_verbosity(0));
    }

    #[test]
    fn direct_answer_synthesis_waits_for_info_verbosity() {
        let direct_answer = TurnEvent::SynthesisReady {
            grounded: false,
            citations: Vec::new(),
            insufficient_evidence: false,
        };
        let grounded_answer = TurnEvent::SynthesisReady {
            grounded: true,
            citations: vec!["src/main.rs".to_string()],
            insufficient_evidence: false,
        };

        assert!(!direct_answer.is_visible_at_verbosity(0));
        assert!(direct_answer.is_visible_at_verbosity(1));
        assert!(grounded_answer.is_visible_at_verbosity(0));
        assert!(!direct_answer.allows_pace_promotion());
    }

    #[test]
    fn harness_state_does_not_allow_pace_promotion() {
        let event = TurnEvent::HarnessState {
            snapshot: HarnessSnapshot::intervening(HarnessChamber::Governor, "fallback"),
        };

        assert!(!event.allows_pace_promotion());
    }
}
