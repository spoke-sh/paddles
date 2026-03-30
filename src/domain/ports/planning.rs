use super::context_gathering::{
    EvidenceItem, PlannerTraceMetadata, RetrievalMode, RetrievalStrategy,
};
use crate::domain::model::{
    ConversationThread, ThreadCandidate, ThreadDecision, TraceBranch, TraceBranchId,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[async_trait]
pub trait RecursivePlanner: Send + Sync {
    fn capability(&self) -> PlannerCapability;

    async fn derive_interpretation_context(
        &self,
        request: &InterpretationRequest,
    ) -> Result<InterpretationContext, anyhow::Error>;

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
    pub tool_hints: Vec<InterpretationToolHint>,
    pub decision_framework: InterpretationDecisionFramework,
}

impl InterpretationContext {
    pub fn is_empty(&self) -> bool {
        self.summary.trim().is_empty()
            && self.documents.is_empty()
            && self.tool_hints.is_empty()
            && self.decision_framework.procedures.is_empty()
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
        if !self.tool_hints.is_empty() {
            sections.push("--- Tool Hints ---".to_string());
            sections.extend(self.tool_hints.iter().map(|hint| {
                format!(
                    "- {} ({}) — {}",
                    hint.action.summary(),
                    hint.source,
                    hint.note
                )
            }));
        }
        if !self.decision_framework.procedures.is_empty() {
            sections.push("--- Decision Framework ---".to_string());
            sections.extend(self.decision_framework.procedures.iter().map(|procedure| {
                format!(
                    "- {} ({}) — {} [{} step(s)]",
                    procedure.label,
                    procedure.source,
                    procedure.purpose,
                    procedure.steps.len()
                )
            }));
        }
        sections.join("\n\n")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterpretationRequest {
    pub user_prompt: String,
    pub workspace_root: PathBuf,
    pub operator_memory: Vec<OperatorMemoryDocument>,
}

impl InterpretationRequest {
    pub fn new(
        user_prompt: impl Into<String>,
        workspace_root: impl Into<PathBuf>,
        operator_memory: Vec<OperatorMemoryDocument>,
    ) -> Self {
        Self {
            user_prompt: user_prompt.into(),
            workspace_root: workspace_root.into(),
            operator_memory,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorMemoryDocument {
    pub path: PathBuf,
    pub source: String,
    pub contents: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterpretationDocument {
    pub source: String,
    pub excerpt: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterpretationToolHint {
    pub source: String,
    pub action: WorkspaceAction,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InterpretationDecisionFramework {
    pub procedures: Vec<InterpretationProcedure>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterpretationProcedure {
    pub source: String,
    pub label: String,
    pub purpose: String,
    pub steps: Vec<InterpretationProcedureStep>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterpretationProcedureStep {
    pub index: usize,
    pub action: WorkspaceAction,
    pub note: String,
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
            max_steps: 6,
            max_branch_factor: 3,
            max_evidence_items: 8,
            max_reads: 3,
            max_inspects: 6,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum WorkspaceAction {
    Search {
        query: String,
        mode: RetrievalMode,
        strategy: RetrievalStrategy,
        #[serde(default)]
        intent: Option<String>,
    },
    ListFiles {
        #[serde(default)]
        pattern: Option<String>,
    },
    Read {
        path: String,
    },
    Inspect {
        command: String,
    },
    Shell {
        command: String,
    },
    Diff {
        #[serde(default)]
        path: Option<String>,
    },
    WriteFile {
        path: String,
        content: String,
    },
    ReplaceInFile {
        path: String,
        old: String,
        new: String,
        #[serde(default)]
        replace_all: bool,
    },
    ApplyPatch {
        patch: String,
    },
}

impl WorkspaceAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Search { .. } => "search",
            Self::ListFiles { .. } => "list_files",
            Self::Read { .. } => "read",
            Self::Inspect { .. } => "inspect",
            Self::Shell { .. } => "shell",
            Self::Diff { .. } => "diff",
            Self::WriteFile { .. } => "write_file",
            Self::ReplaceInFile { .. } => "replace_in_file",
            Self::ApplyPatch { .. } => "apply_patch",
        }
    }

    pub fn summary(&self) -> String {
        match self {
            Self::Search {
                query,
                mode,
                strategy,
                ..
            } => format!("search `{query}` [{} / {}]", mode.label(), strategy.label()),
            Self::ListFiles { pattern } => match pattern {
                Some(pattern) if !pattern.trim().is_empty() => {
                    format!("list files matching `{pattern}`")
                }
                _ => "list files".to_string(),
            },
            Self::Read { path } => format!("read `{path}`"),
            Self::Inspect { command } => format!("inspect `{command}`"),
            Self::Shell { command } => command.clone(),
            Self::Diff { path } => match path {
                Some(path) if !path.trim().is_empty() => format!("diff `{path}`"),
                _ => "git diff --no-ext-diff".to_string(),
            },
            Self::WriteFile { path, .. } => format!("write `{path}`"),
            Self::ReplaceInFile { path, .. } => format!("replace text in `{path}`"),
            Self::ApplyPatch { .. } => "git apply --whitespace=nowarn -".to_string(),
        }
    }

    /// Human-readable description of this action for event and trace output.
    pub fn describe(&self) -> String {
        match self {
            Self::Search { query, intent, .. } => match intent {
                Some(intent) => format!("search workspace for `{query}` ({intent})"),
                None => format!("search workspace for `{query}`"),
            },
            Self::ListFiles { pattern } => match pattern {
                Some(pattern) if !pattern.trim().is_empty() => {
                    format!("list files matching `{pattern}`")
                }
                _ => "list workspace files".to_string(),
            },
            Self::Read { path } => format!("read `{path}`"),
            Self::Inspect { command } => format!("inspect `{command}`"),
            Self::Shell { command } => command.clone(),
            Self::Diff { path } => match path {
                Some(path) if !path.trim().is_empty() => {
                    format!("git diff --no-ext-diff -- {path}")
                }
                _ => "git diff --no-ext-diff".to_string(),
            },
            Self::WriteFile { path, .. } => format!("write `{path}`"),
            Self::ReplaceInFile { path, .. } => format!("replace text in `{path}`"),
            Self::ApplyPatch { .. } => "git apply --whitespace=nowarn -".to_string(),
        }
    }
}

/// The first bounded action the planner may choose for a turn.
///
/// This contract is intentionally generic across repositories and evidence
/// domains. The controller remains responsible for validating read-only
/// inspect commands, enforcing budgets, and failing closed when the selected
/// action cannot be executed safely.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitialAction {
    Answer,
    Workspace {
        action: WorkspaceAction,
    },
    Refine {
        query: String,
        mode: RetrievalMode,
        strategy: RetrievalStrategy,
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
            Self::Workspace { action } => action.label(),
            Self::Refine { .. } => "refine",
            Self::Branch { .. } => "branch",
            Self::Stop { .. } => "stop",
        }
    }

    pub fn summary(&self) -> String {
        match self {
            Self::Answer => "answer directly".to_string(),
            Self::Workspace { action } => action.summary(),
            Self::Refine {
                query,
                mode,
                strategy,
                ..
            } => format!(
                "refine toward `{query}` [{} / {}]",
                mode.label(),
                strategy.label()
            ),
            Self::Branch { branches, .. } => format!("branch into {}", branches.join(" | ")),
            Self::Stop { reason } => format!("stop ({reason})"),
        }
    }

    pub fn as_planner_action(&self) -> Option<PlannerAction> {
        match self {
            Self::Answer => None,
            Self::Workspace { action } => Some(PlannerAction::Workspace {
                action: action.clone(),
            }),
            Self::Refine {
                query,
                mode,
                strategy,
                rationale,
            } => Some(PlannerAction::Refine {
                query: query.clone(),
                mode: *mode,
                strategy: *strategy,
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
    Workspace {
        action: WorkspaceAction,
    },
    Refine {
        query: String,
        mode: RetrievalMode,
        strategy: RetrievalStrategy,
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
            Self::Workspace { action } => action.label(),
            Self::Refine { .. } => "refine",
            Self::Branch { .. } => "branch",
            Self::Stop { .. } => "stop",
        }
    }

    pub fn summary(&self) -> String {
        match self {
            Self::Workspace { action } => action.summary(),
            Self::Refine {
                query,
                mode,
                strategy,
                ..
            } => format!(
                "refine toward `{query}` [{} / {}]",
                mode.label(),
                strategy.label()
            ),
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
        InitialAction, InterpretationContext, InterpretationDecisionFramework,
        InterpretationDocument, InterpretationProcedure, InterpretationProcedureStep,
        InterpretationToolHint, PlannerAction, PlannerBudget, RetrievalMode, RetrievalStrategy,
        ThreadDecisionRequest, WorkspaceAction,
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
            tool_hints: vec![InterpretationToolHint {
                source: "INSTRUCTIONS.md".to_string(),
                action: WorkspaceAction::Inspect {
                    command: "keel mission next".to_string(),
                },
                note: "Inspect current demand on the board.".to_string(),
            }],
            decision_framework: InterpretationDecisionFramework {
                procedures: vec![InterpretationProcedure {
                    source: "INSTRUCTIONS.md".to_string(),
                    label: "Inspect".to_string(),
                    purpose: "Inspect current demand on the board.".to_string(),
                    steps: vec![InterpretationProcedureStep {
                        index: 0,
                        action: WorkspaceAction::Inspect {
                            command: "keel mission next".to_string(),
                        },
                        note: "Read current demand.".to_string(),
                    }],
                }],
            },
        };

        let rendered = context.render();
        assert!(rendered.contains("operator memory"));
        assert!(rendered.contains("AGENTS.md"));
        assert!(rendered.contains("Tool Hints"));
        assert!(rendered.contains("keel mission next"));
        assert!(rendered.contains("Decision Framework"));
        assert!(rendered.contains("Inspect"));
        assert_eq!(context.sources(), vec!["AGENTS.md".to_string()]);
    }

    #[test]
    fn planner_budget_has_bounded_defaults() {
        let budget = PlannerBudget::default();
        assert_eq!(budget.max_steps, 6);
        assert_eq!(budget.max_branch_factor, 3);
        assert_eq!(budget.max_evidence_items, 8);
        assert_eq!(budget.max_inspects, 6);
    }

    #[test]
    fn planner_action_reports_human_readable_summary() {
        let action = PlannerAction::Workspace {
            action: WorkspaceAction::Search {
                query: "memory reload".to_string(),
                mode: RetrievalMode::Linear,
                strategy: RetrievalStrategy::Lexical,
                intent: None,
            },
        };

        assert_eq!(action.label(), "search");
        assert!(action.summary().contains("memory reload"));
        assert!(!action.is_terminal());
    }

    #[test]
    fn initial_action_reports_human_readable_summary() {
        let action = InitialAction::Workspace {
            action: WorkspaceAction::Inspect {
                command: "git status".to_string(),
            },
        };

        assert_eq!(action.label(), "inspect");
        assert!(action.summary().contains("git status"));
        assert_eq!(
            action.as_planner_action(),
            Some(PlannerAction::Workspace {
                action: WorkspaceAction::Inspect {
                    command: "git status".to_string()
                }
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
