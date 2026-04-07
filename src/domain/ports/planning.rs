use super::context_gathering::{
    EvidenceItem, PlannerTraceMetadata, RetrievalMode, RetrievalStrategy, RetrieverOption,
};
use super::context_resolution::ContextResolver;
pub use crate::domain::model::{
    CompactionPlan, CompactionRequest, ConversationThread, GuidanceCategory,
    InterpretationConflict, InterpretationContext, InterpretationCoverageConfidence,
    InterpretationDecisionFramework, InterpretationDocument, InterpretationProcedure,
    InterpretationProcedureStep, InterpretationToolHint, ThreadCandidate, ThreadDecision,
    TraceBranch, TraceBranchId, WorkspaceAction,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::domain::model::TurnEventSink;
use std::sync::Arc;

#[async_trait]
pub trait RecursivePlanner: Send + Sync {
    fn capability(&self) -> PlannerCapability;

    async fn derive_interpretation_context(
        &self,
        request: &InterpretationRequest,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<InterpretationContext, anyhow::Error>;

    async fn select_initial_action(
        &self,
        request: &PlannerRequest,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<InitialActionDecision, anyhow::Error>;

    async fn select_next_action(
        &self,
        request: &PlannerRequest,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<PlannerDecision, anyhow::Error>;

    async fn select_thread_decision(
        &self,
        request: &ThreadDecisionRequest,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<ThreadDecision, anyhow::Error>;

    /// Evaluate context artifacts for relevance and produce a compaction plan.
    async fn assess_context_relevance(
        &self,
        request: &CompactionRequest,
    ) -> Result<CompactionPlan, anyhow::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlannerCapability {
    Available,
    Unsupported { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum RefinementTriggerSource {
    #[default]
    #[serde(rename = "premise_challenge", alias = "evidence_pressure")]
    PremiseChallenge,
    StaleLoopState,
    Manual,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefinementTrigger {
    pub id: String,
    pub source: RefinementTriggerSource,
    pub min_evidence_items: usize,
    pub min_steps_without_new_evidence: usize,
}

impl Default for RefinementTrigger {
    fn default() -> Self {
        Self {
            id: "trigger:premise-challenge-v1".to_string(),
            source: RefinementTriggerSource::PremiseChallenge,
            min_evidence_items: 1,
            min_steps_without_new_evidence: 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefinementPolicy {
    pub id: String,
    pub enabled: bool,
    pub trigger: RefinementTrigger,
    pub max_refinements_per_turn: usize,
    pub cooldown_steps: usize,
    pub oscillation_signature_window: usize,
    pub signature_history_limit: usize,
}

impl Default for RefinementPolicy {
    fn default() -> Self {
        Self {
            id: "policy:context-refine-v1".to_string(),
            enabled: true,
            trigger: RefinementTrigger::default(),
            max_refinements_per_turn: 1,
            cooldown_steps: 2,
            oscillation_signature_window: 3,
            signature_history_limit: 6,
        }
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
pub struct PlannerBudget {
    pub max_steps: usize,
    pub max_branch_factor: usize,
    pub max_evidence_items: usize,
    pub max_reads: usize,
    pub max_inspects: usize,
    pub max_searches: usize,
}

impl Default for PlannerBudget {
    fn default() -> Self {
        Self {
            max_steps: 12,
            max_branch_factor: 3,
            max_evidence_items: 12,
            max_reads: 6,
            max_inspects: 6,
            max_searches: 4,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlannerRequest {
    pub user_prompt: String,
    pub workspace_root: PathBuf,
    pub interpretation: InterpretationContext,
    pub recent_turns: Vec<String>,
    pub recent_thread_summary: Option<String>,
    pub runtime_notes: Vec<String>,
    pub loop_state: PlannerLoopState,
    pub budget: PlannerBudget,
    pub resolver: Option<Arc<dyn ContextResolver>>,
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
            recent_thread_summary: None,
            runtime_notes: Vec::new(),
            loop_state: PlannerLoopState::default(),
            budget,
            resolver: None,
        }
    }

    pub fn with_resolver(mut self, resolver: Arc<dyn ContextResolver>) -> Self {
        self.resolver = Some(resolver);
        self
    }

    pub fn with_recent_turns(mut self, recent_turns: Vec<String>) -> Self {
        self.recent_turns = recent_turns;
        self
    }

    pub fn with_recent_thread_summary(mut self, recent_thread_summary: Option<String>) -> Self {
        self.recent_thread_summary = recent_thread_summary;
        self
    }

    pub fn with_runtime_notes(mut self, runtime_notes: Vec<String>) -> Self {
        self.runtime_notes = runtime_notes;
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
    pub refinement_count: usize,
    pub last_refinement_step: Option<usize>,
    pub refinement_signatures: Vec<String>,
    pub refinement_policy: RefinementPolicy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroundingDomain {
    Repository,
    External,
    Mixed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundingRequirement {
    pub domain: GroundingDomain,
    #[serde(default)]
    pub reason: Option<String>,
}

impl GroundingRequirement {
    pub fn requires_external(&self) -> bool {
        matches!(
            self.domain,
            GroundingDomain::External | GroundingDomain::Mixed
        )
    }
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
        retrievers: Vec<RetrieverOption>,
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
                retrievers,
                ..
            } => format!(
                "refine toward `{query}` [{} / {}{}]",
                mode.label(),
                strategy.label(),
                format_retriever_suffix(retrievers)
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
                retrievers,
                rationale,
            } => Some(PlannerAction::Refine {
                query: query.clone(),
                mode: *mode,
                strategy: *strategy,
                retrievers: retrievers.clone(),
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
    pub answer: Option<String>,
    pub edit: InitialEditInstruction,
    pub grounding: Option<GroundingRequirement>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InitialEditInstruction {
    pub known_edit: bool,
    pub candidate_files: Vec<String>,
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
        retrievers: Vec<RetrieverOption>,
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
                retrievers,
                ..
            } => format!(
                "refine toward `{query}` [{} / {}{}]",
                mode.label(),
                strategy.label(),
                format_retriever_suffix(retrievers)
            ),
            Self::Branch { branches, .. } => {
                format!("branch into {}", branches.join(" | "))
            }
            Self::Stop { reason } => format!("stop ({reason})"),
        }
    }

    pub fn target_query(&self) -> Option<String> {
        match self {
            Self::Workspace { action } => match action {
                WorkspaceAction::Search { query, .. } => Some(query.clone()),
                WorkspaceAction::Read { path } => Some(path.clone()),
                WorkspaceAction::Inspect { command } => Some(command.clone()),
                WorkspaceAction::Shell { command } => Some(command.clone()),
                WorkspaceAction::ListFiles { pattern } => pattern.clone(),
                WorkspaceAction::Diff { path } => path.clone(),
                WorkspaceAction::WriteFile { path, .. } => Some(path.clone()),
                WorkspaceAction::ReplaceInFile { path, .. } => Some(path.clone()),
                WorkspaceAction::ApplyPatch { .. } => None,
            },
            Self::Refine { query, .. } => Some(query.clone()),
            Self::Branch { branches, .. } => Some(branches.join(" | ")),
            Self::Stop { reason } => Some(reason.clone()),
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
    pub answer: Option<String>,
    pub edit: InitialEditInstruction,
    pub grounding: Option<GroundingRequirement>,
}

fn format_retriever_suffix(retrievers: &[RetrieverOption]) -> String {
    if retrievers.is_empty() {
        String::new()
    } else {
        format!(
            "; retrievers={}",
            retrievers
                .iter()
                .map(RetrieverOption::label)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GuidanceCategory, InitialAction, InterpretationContext, InterpretationDecisionFramework,
        InterpretationDocument, InterpretationProcedure, InterpretationProcedureStep,
        InterpretationToolHint, PlannerAction, PlannerBudget, PlannerLoopState, RefinementPolicy,
        RefinementTrigger, RefinementTriggerSource, RetrievalMode, RetrievalStrategy,
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
                category: GuidanceCategory::Rule,
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
            ..InterpretationContext::default()
        };

        let rendered = context.render();
        assert!(rendered.contains("operator memory"));
        assert!(rendered.contains("AGENTS.md"));
        assert!(rendered.contains("Tool Hints"));
        assert!(rendered.contains("keel mission next"));
        assert!(rendered.contains("Decision Framework"));
        assert!(rendered.contains("Inspect"));
        assert_eq!(
            context.sources(),
            vec!["AGENTS.md".to_string(), "INSTRUCTIONS.md".to_string()]
        );
    }

    #[test]
    fn planner_budget_has_bounded_defaults() {
        let budget = PlannerBudget::default();
        assert_eq!(budget.max_steps, 12);
        assert_eq!(budget.max_branch_factor, 3);
        assert_eq!(budget.max_evidence_items, 12);
        assert_eq!(budget.max_inspects, 6);
        assert_eq!(budget.max_searches, 4);
    }

    #[test]
    fn planner_action_reports_human_readable_summary() {
        let action = PlannerAction::Workspace {
            action: WorkspaceAction::Search {
                query: "memory reload".to_string(),
                mode: RetrievalMode::Linear,
                strategy: RetrievalStrategy::Lexical,
                retrievers: Vec::new(),
                intent: None,
            },
        };

        assert_eq!(action.label(), "search");
        assert!(action.summary().contains("memory reload"));
        assert!(!action.is_terminal());
    }

    #[test]
    fn refinement_types_define_stable_defaults() {
        let trigger = RefinementTrigger::default();
        let policy = RefinementPolicy::default();

        assert_eq!(trigger.id, "trigger:premise-challenge-v1");
        assert_eq!(trigger.source, RefinementTriggerSource::PremiseChallenge);
        assert_eq!(trigger.min_evidence_items, 1);
        assert_eq!(trigger.min_steps_without_new_evidence, 2);
        assert_eq!(policy.id, "policy:context-refine-v1");
        assert!(policy.enabled);
        assert!(policy.max_refinements_per_turn > 0);
    }

    #[test]
    fn planner_loop_state_defaults_include_refinement_fields() {
        let state = PlannerLoopState::default();

        assert_eq!(state.refinement_count, 0);
        assert!(state.last_refinement_step.is_none());
        assert!(state.refinement_signatures.is_empty());
        assert_eq!(state.refinement_policy, RefinementPolicy::default());
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
