use super::context_quality::ContextPressure;
use super::interpretation::InterpretationContext;
use serde::Serialize;
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
    ContextPressure {
        pressure: ContextPressure,
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
            Self::PlannerSummary { .. } => "planner_summary",
            Self::RefinementApplied { .. } => "refinement_applied",
            Self::ContextAssembly { .. } => "context_assembly",
            Self::ToolCalled { .. } => "tool_called",
            Self::ToolFinished { .. } => "tool_finished",
            Self::Fallback { .. } => "fallback",
            Self::ContextPressure { .. } => "context_pressure",
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
            | Self::PlannerSummary { .. }
            | Self::ContextAssembly { .. }
            | Self::ContextPressure { .. }
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
}

pub trait TurnEventSink: Send + Sync {
    fn emit(&self, event: TurnEvent);
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
