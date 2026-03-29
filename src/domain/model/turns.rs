use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TurnEvent {
    IntentClassified {
        intent: TurnIntent,
    },
    InterpretationContext {
        summary: String,
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
    SynthesisReady {
        grounded: bool,
        citations: Vec<String>,
        insufficient_evidence: bool,
    },
}

pub trait TurnEventSink: Send + Sync {
    fn emit(&self, event: TurnEvent);
}

#[derive(Default)]
pub struct NullTurnEventSink;

impl TurnEventSink for NullTurnEventSink {
    fn emit(&self, _event: TurnEvent) {}
}
