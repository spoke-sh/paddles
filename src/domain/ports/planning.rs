use super::context_gathering::EvidenceItem;
use async_trait::async_trait;
use std::path::PathBuf;

#[async_trait]
pub trait RecursivePlanner: Send + Sync {
    fn capability(&self) -> PlannerCapability;

    async fn select_next_action(
        &self,
        request: &PlannerRequest,
    ) -> Result<PlannerDecision, anyhow::Error>;
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

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PlannerLoopState {
    pub steps: Vec<PlannerStepRecord>,
    pub evidence_items: Vec<EvidenceItem>,
    pub notes: Vec<String>,
    pub pending_branches: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerStepRecord {
    pub sequence: usize,
    pub action: PlannerAction,
    pub outcome: String,
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
    use super::{InterpretationContext, InterpretationDocument, PlannerAction, PlannerBudget};

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
}
