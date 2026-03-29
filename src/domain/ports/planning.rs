use super::context_gathering::{EvidenceItem, PlannerTraceMetadata};
use crate::domain::model::{
    ConversationThread, ThreadCandidate, ThreadDecision, TraceBranch, TraceBranchId,
};
use async_trait::async_trait;
use std::path::PathBuf;

#[async_trait]
pub trait RecursivePlanner: Send + Sync {
    fn capability(&self) -> PlannerCapability;

    async fn select_initial_action(
        &self,
        request: &PlannerRequest,
    ) -> Result<InitialActionDecision, anyhow::Error>;

    async fn select_next_action(
        &self,
        request: &PlannerRequest,
    ) -> Result<PlannerDecision, anyhow::Error>;

    async fn select_thread_decision(
        &self,
        request: &ThreadDecisionRequest,
    ) -> Result<ThreadDecision, anyhow::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlannerCapability {
    Available,
    Unsupported { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InterpretationContext {
    pub summary: String,
    pub documents: Vec<InterpretationDocument>,
}

impl InterpretationContext {
    pub fn is_empty(&self) -> bool {
        self.summary.trim().is_empty() && self.documents.is_empty()
    }

    pub fn sources(&self) -> Vec<String> {
        self.documents
            .iter()
            .map(|document| document.source.clone())
            .collect()
    }

    pub fn render(&self) -> String {
        if self.is_empty() {
            return "No operator interpretation context was assembled.".to_string();
        }

        let mut sections = vec![self.summary.trim().to_string()];
        for document in &self.documents {
            sections.push(format!(
                "--- {} ---\n{}",
                document.source,
                document.excerpt.trim()
            ));
        }
        sections.join("\n\n")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterpretationDocument {
    pub source: String,
    pub excerpt: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerBudget {
    pub max_steps: usize,
    pub max_branch_factor: usize,
    pub max_evidence_items: usize,
    pub max_reads: usize,
    pub max_inspects: usize,
}

impl Default for PlannerBudget {
    fn default() -> Self {
        Self {
            max_steps: 4,
            max_branch_factor: 3,
            max_evidence_items: 8,
            max_reads: 3,
            max_inspects: 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerRequest {
    pub user_prompt: String,
    pub workspace_root: PathBuf,
    pub interpretation: InterpretationContext,
    pub recent_turns: Vec<String>,
    pub loop_state: PlannerLoopState,
    pub budget: PlannerBudget,
}

impl PlannerRequest {
    pub fn new(
        user_prompt: impl Into<String>,
        workspace_root: impl Into<PathBuf>,
        interpretation: InterpretationContext,
        budget: PlannerBudget,
    ) -> Self {
        Self {
            user_prompt: user_prompt.into(),
            workspace_root: workspace_root.into(),
            interpretation,
            recent_turns: Vec::new(),
            loop_state: PlannerLoopState::default(),
            budget,
        }
    }

    pub fn with_recent_turns(mut self, recent_turns: Vec<String>) -> Self {
        self.recent_turns = recent_turns;
        self
    }

    pub fn with_loop_state(mut self, loop_state: PlannerLoopState) -> Self {
        self.loop_state = loop_state;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThreadDecisionRequest {
    pub workspace_root: PathBuf,
    pub interpretation: InterpretationContext,
    pub recent_turns: Vec<String>,
    pub active_thread: ConversationThread,
    pub known_threads: Vec<ConversationThread>,
    pub candidate: ThreadCandidate,
    pub recent_thread_summary: Option<String>,
}

impl ThreadDecisionRequest {
    pub fn new(
        workspace_root: impl Into<PathBuf>,
        interpretation: InterpretationContext,
        active_thread: ConversationThread,
        candidate: ThreadCandidate,
    ) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            interpretation,
            recent_turns: Vec::new(),
            active_thread,
            known_threads: Vec::new(),
            candidate,
            recent_thread_summary: None,
        }
    }

    pub fn with_recent_turns(mut self, recent_turns: Vec<String>) -> Self {
        self.recent_turns = recent_turns;
        self
    }

    pub fn with_known_threads(mut self, known_threads: Vec<ConversationThread>) -> Self {
        self.known_threads = known_threads;
        self
    }

    pub fn with_recent_thread_summary(mut self, recent_thread_summary: Option<String>) -> Self {
        self.recent_thread_summary = recent_thread_summary;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PlannerLoopState {
    pub steps: Vec<PlannerStepRecord>,
    pub evidence_items: Vec<EvidenceItem>,
    pub notes: Vec<String>,
    pub pending_branches: Vec<TraceBranch>,
    pub latest_gatherer_trace: Option<PlannerTraceMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerStepRecord {
    pub step_id: String,
    pub sequence: usize,
    pub branch_id: Option<TraceBranchId>,
    pub action: PlannerAction,
    pub outcome: String,
}

/// The first bounded action the planner may choose for a turn.
///
/// This contract is intentionally generic across repositories and evidence
/// domains. The controller remains responsible for validating read-only
/// inspect commands, enforcing budgets, and failing closed when the selected
/// action cannot be executed safely.
///
/// `Tool` exists as a transitional bridge into the existing deterministic tool
/// runtime while the harness still separates top-level action selection from
/// lower-level tool execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitialAction {
    Answer,
    Tool,
    Search {
        query: String,
        intent: Option<String>,
    },
    Read {
        path: String,
    },
    Inspect {
        command: String,
    },
    Refine {
        query: String,
        rationale: Option<String>,
    },
    Branch {
        branches: Vec<String>,
        rationale: Option<String>,
    },
    Stop {
        reason: String,
    },
}

impl InitialAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Answer => "answer",
            Self::Tool => "tool",
            Self::Search { .. } => "search",
            Self::Read { .. } => "read",
            Self::Inspect { .. } => "inspect",
            Self::Refine { .. } => "refine",
            Self::Branch { .. } => "branch",
            Self::Stop { .. } => "stop",
        }
    }

    pub fn summary(&self) -> String {
        match self {
            Self::Answer => "answer directly".to_string(),
            Self::Tool => "use deterministic tool execution".to_string(),
            Self::Search { query, .. } => format!("search `{query}`"),
            Self::Read { path } => format!("read `{path}`"),
            Self::Inspect { command } => format!("inspect `{command}`"),
            Self::Refine { query, .. } => format!("refine toward `{query}`"),
            Self::Branch { branches, .. } => format!("branch into {}", branches.join(" | ")),
            Self::Stop { reason } => format!("stop ({reason})"),
        }
    }

    pub fn as_planner_action(&self) -> Option<PlannerAction> {
        match self {
            Self::Answer | Self::Tool => None,
            Self::Search { query, intent } => Some(PlannerAction::Search {
                query: query.clone(),
                intent: intent.clone(),
            }),
            Self::Read { path } => Some(PlannerAction::Read { path: path.clone() }),
            Self::Inspect { command } => Some(PlannerAction::Inspect {
                command: command.clone(),
            }),
            Self::Refine { query, rationale } => Some(PlannerAction::Refine {
                query: query.clone(),
                rationale: rationale.clone(),
            }),
            Self::Branch {
                branches,
                rationale,
            } => Some(PlannerAction::Branch {
                branches: branches.clone(),
                rationale: rationale.clone(),
            }),
            Self::Stop { reason } => Some(PlannerAction::Stop {
                reason: reason.clone(),
            }),
        }
    }
}

/// A planner-selected first action paired with the model's routing rationale.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitialActionDecision {
    pub action: InitialAction,
    pub rationale: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlannerAction {
    Search {
        query: String,
        intent: Option<String>,
    },
    Read {
        path: String,
    },
    Inspect {
        command: String,
    },
    Refine {
        query: String,
        rationale: Option<String>,
    },
    Branch {
        branches: Vec<String>,
        rationale: Option<String>,
    },
    Stop {
        reason: String,
    },
}

impl PlannerAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Search { .. } => "search",
            Self::Read { .. } => "read",
            Self::Inspect { .. } => "inspect",
            Self::Refine { .. } => "refine",
            Self::Branch { .. } => "branch",
            Self::Stop { .. } => "stop",
        }
    }

    pub fn summary(&self) -> String {
        match self {
            Self::Search { query, .. } => format!("search `{query}`"),
            Self::Read { path } => format!("read `{path}`"),
            Self::Inspect { command } => format!("inspect `{command}`"),
            Self::Refine { query, .. } => format!("refine toward `{query}`"),
            Self::Branch { branches, .. } => {
                format!("branch into {}", branches.join(" | "))
            }
            Self::Stop { reason } => format!("stop ({reason})"),
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Stop { .. })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerDecision {
    pub action: PlannerAction,
    pub rationale: String,
}

#[cfg(test)]
mod tests {
    use super::{
        InitialAction, InterpretationContext, InterpretationDocument, PlannerAction, PlannerBudget,
        ThreadDecisionRequest,
    };
    use crate::domain::model::{
        ConversationThread, ConversationThreadRef, ConversationThreadStatus, ThreadCandidate,
        ThreadCandidateId,
    };

    #[test]
    fn interpretation_context_renders_sources() {
        let context = InterpretationContext {
            summary: "operator memory".to_string(),
            documents: vec![InterpretationDocument {
                source: "AGENTS.md".to_string(),
                excerpt: "guidance".to_string(),
            }],
        };

        let rendered = context.render();
        assert!(rendered.contains("operator memory"));
        assert!(rendered.contains("AGENTS.md"));
        assert_eq!(context.sources(), vec!["AGENTS.md".to_string()]);
    }

    #[test]
    fn planner_budget_has_bounded_defaults() {
        let budget = PlannerBudget::default();
        assert_eq!(budget.max_steps, 4);
        assert_eq!(budget.max_branch_factor, 3);
        assert_eq!(budget.max_evidence_items, 8);
    }

    #[test]
    fn planner_action_reports_human_readable_summary() {
        let action = PlannerAction::Search {
            query: "memory reload".to_string(),
            intent: None,
        };

        assert_eq!(action.label(), "search");
        assert!(action.summary().contains("memory reload"));
        assert!(!action.is_terminal());
    }

    #[test]
    fn initial_action_reports_human_readable_summary() {
        let action = InitialAction::Inspect {
            command: "git status".to_string(),
        };

        assert_eq!(action.label(), "inspect");
        assert!(action.summary().contains("git status"));
        assert_eq!(
            action.as_planner_action(),
            Some(PlannerAction::Inspect {
                command: "git status".to_string()
            })
        );
    }

    #[test]
    fn thread_decision_request_keeps_generic_thread_state() {
        let request = ThreadDecisionRequest::new(
            ".",
            InterpretationContext::default(),
            ConversationThread {
                thread_ref: ConversationThreadRef::Mainline,
                label: "mainline".to_string(),
                parent: None,
                status: ConversationThreadStatus::Active,
            },
            ThreadCandidate {
                candidate_id: ThreadCandidateId::new("candidate-1").expect("candidate"),
                prompt: "Steer the current work".to_string(),
                captured_from_turn_id: None,
                active_thread: ConversationThreadRef::Mainline,
                captured_sequence: 1,
            },
        );

        assert_eq!(request.active_thread.label, "mainline");
        assert_eq!(request.candidate.prompt, "Steer the current work");
        assert!(request.recent_thread_summary.is_none());
    }
}
