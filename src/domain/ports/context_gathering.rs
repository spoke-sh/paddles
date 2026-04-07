use async_trait::async_trait;
use paddles_conversation::ContextLocator;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Port for specialized context-gathering subagents.
#[async_trait]
pub trait ContextGatherer: Send + Sync {
    /// Report whether this gatherer is actually usable in the current runtime.
    fn capability(&self) -> GathererCapability;

    /// Report whether the gatherer can satisfy a concrete retrieval plan right now.
    fn capability_for_planning(&self, _planning: &PlannerConfig) -> GathererCapability {
        self.capability()
    }

    /// Gather ranked evidence for a retrieval-heavy request.
    async fn gather_context(
        &self,
        request: &ContextGatherRequest,
    ) -> Result<ContextGatherResult, anyhow::Error>;

    /// Optionally wire a per-turn event sink for progress reporting.
    fn set_event_sink(
        &self,
        _sink: Option<std::sync::Arc<dyn crate::domain::model::TurnEventSink>>,
    ) {
    }
}

/// Request sent to a context-gathering lane before final synthesis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextGatherRequest {
    pub user_query: String,
    pub workspace_root: PathBuf,
    pub intent_reason: String,
    pub budget: EvidenceBudget,
    pub planning: PlannerConfig,
    pub prior_context: Vec<String>,
}

impl ContextGatherRequest {
    pub fn new(
        user_query: impl Into<String>,
        workspace_root: impl Into<PathBuf>,
        intent_reason: impl Into<String>,
        budget: EvidenceBudget,
    ) -> Self {
        Self {
            user_query: user_query.into(),
            workspace_root: workspace_root.into(),
            intent_reason: intent_reason.into(),
            budget,
            planning: PlannerConfig::default(),
            prior_context: Vec::new(),
        }
    }

    pub fn with_planning(mut self, planning: PlannerConfig) -> Self {
        self.planning = planning;
        self
    }

    pub fn with_prior_context(mut self, prior_context: Vec<String>) -> Self {
        self.prior_context = prior_context;
        self
    }
}

/// Size limits for evidence returned to the synthesizer lane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceBudget {
    pub max_items: usize,
    pub max_summary_chars: usize,
    pub max_snippet_chars: usize,
}

impl Default for EvidenceBudget {
    fn default() -> Self {
        Self {
            max_items: 8,
            max_summary_chars: 1_200,
            max_snippet_chars: 600,
        }
    }
}

/// Planner controls attached to a gather request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerConfig {
    pub mode: RetrievalMode,
    pub retrieval_strategy: RetrievalStrategy,
    pub retrievers: Vec<RetrieverOption>,
    pub planner_strategy: PlannerStrategyKind,
    pub profile: Option<String>,
    pub step_limit: usize,
    pub retained_artifact_limit: usize,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            mode: RetrievalMode::default(),
            retrieval_strategy: RetrievalStrategy::default(),
            retrievers: Vec::new(),
            planner_strategy: PlannerStrategyKind::Heuristic,
            profile: None,
            step_limit: 3,
            retained_artifact_limit: 5,
        }
    }
}

impl PlannerConfig {
    pub fn with_mode(mut self, mode: RetrievalMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_retrieval_strategy(mut self, strategy: RetrievalStrategy) -> Self {
        self.retrieval_strategy = strategy;
        self
    }

    pub fn with_retrievers(mut self, retrievers: Vec<RetrieverOption>) -> Self {
        self.retrievers = retrievers;
        self
    }

    pub fn with_strategy(mut self, strategy: PlannerStrategyKind) -> Self {
        self.planner_strategy = strategy;
        self
    }

    pub fn with_profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = Some(profile.into());
        self
    }

    pub fn with_step_limit(mut self, step_limit: usize) -> Self {
        self.step_limit = step_limit;
        self
    }

    pub fn with_retained_artifact_limit(mut self, retained_artifact_limit: usize) -> Self {
        self.retained_artifact_limit = retained_artifact_limit;
        self
    }
}

/// Retrieval mode used by a gatherer-backed autonomous planner.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalMode {
    #[default]
    Linear,
    Graph,
}

impl RetrievalMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Graph => "graph",
        }
    }
}

/// Retrieval strategy used by a gatherer-backed search plan.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalStrategy {
    #[default]
    #[serde(alias = "bm25")]
    Lexical,
    Vector,
}

impl RetrievalStrategy {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Lexical => "bm25",
            Self::Vector => "vector",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetrieverOption {
    PathFuzzy,
    SegmentFuzzy,
}

impl RetrieverOption {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PathFuzzy => "path-fuzzy",
            Self::SegmentFuzzy => "segment-fuzzy",
        }
    }
}

/// Capability state for the active gatherer implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GathererCapability {
    Available,
    Warming { reason: String },
    Unsupported { reason: String },
    HarnessRequired { reason: String },
}

/// A ranked evidence item intended for downstream synthesis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceItem {
    pub source: String,
    pub snippet: String,
    pub rationale: String,
    pub rank: usize,
}

/// Planner strategy used by an autonomous gatherer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlannerStrategyKind {
    Heuristic,
    ModelDriven,
}

/// Planner decision metadata exposed to downstream synthesis and telemetry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerDecision {
    pub action: String,
    pub query: Option<String>,
    pub rationale: Option<String>,
    pub next_step_id: Option<String>,
    pub turn_id: Option<String>,
    pub branch_id: Option<String>,
    pub node_id: Option<String>,
    pub target_branch_id: Option<String>,
    pub target_node_id: Option<String>,
    pub edge_id: Option<String>,
    pub edge_kind: Option<PlannerGraphEdgeKind>,
    pub frontier_id: Option<String>,
    pub stop_reason: Option<String>,
}

/// Planner trace metadata for a single autonomous step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerTraceStep {
    pub step_id: String,
    pub sequence: usize,
    pub parent_step_id: Option<String>,
    pub decisions: Vec<PlannerDecision>,
}

/// Retained evidence summary surfaced from planner state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RetainedEvidence {
    pub source: String,
    pub snippet: Option<String>,
    pub rationale: Option<String>,
    pub locator: Option<ContextLocator>,
}

/// Stable branch status surfaced through graph-mode gatherer traces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlannerGraphBranchStatus {
    Pending,
    Active,
    Completed,
    Merged,
    Pruned,
}

impl PlannerGraphBranchStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Merged => "merged",
            Self::Pruned => "pruned",
        }
    }
}

/// Stable edge kind surfaced through graph-mode gatherer traces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlannerGraphEdgeKind {
    Root,
    Child,
    Sibling,
    Merge,
}

impl PlannerGraphEdgeKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Child => "child",
            Self::Sibling => "sibling",
            Self::Merge => "merge",
        }
    }
}

/// Stable graph node metadata preserved from a graph-mode gatherer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerGraphNode {
    pub node_id: String,
    pub branch_id: String,
    pub step_id: String,
    pub parent_step_id: Option<String>,
    pub sequence: usize,
    pub query: Option<String>,
    pub turn_id: Option<String>,
}

/// Stable graph edge metadata preserved from a graph-mode gatherer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerGraphEdge {
    pub edge_id: String,
    pub from_node_id: String,
    pub to_node_id: String,
    pub kind: PlannerGraphEdgeKind,
}

/// Stable frontier metadata preserved from a graph-mode gatherer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerGraphFrontierEntry {
    pub frontier_id: String,
    pub branch_id: String,
    pub node_id: String,
    pub priority: usize,
}

/// Stable branch metadata preserved from a graph-mode gatherer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerGraphBranch {
    pub branch_id: String,
    pub status: PlannerGraphBranchStatus,
    pub head_node_id: String,
    pub retained_artifacts: Vec<RetainedEvidence>,
}

/// Stable graph episode state preserved from a graph-mode gatherer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerGraphEpisode {
    pub root_node_id: Option<String>,
    pub active_branch_id: Option<String>,
    pub frontier: Vec<PlannerGraphFrontierEntry>,
    pub branches: Vec<PlannerGraphBranch>,
    pub nodes: Vec<PlannerGraphNode>,
    pub edges: Vec<PlannerGraphEdge>,
    pub completed: bool,
    pub artifact_ref: Option<String>,
}

/// Planner metadata returned alongside synthesis-ready evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerTraceMetadata {
    pub mode: RetrievalMode,
    pub strategy: PlannerStrategyKind,
    pub profile: Option<String>,
    pub session_id: Option<String>,
    pub completed: bool,
    pub stop_reason: Option<String>,
    pub turn_count: usize,
    pub steps: Vec<PlannerTraceStep>,
    pub retained_artifacts: Vec<RetainedEvidence>,
    pub graph_episode: Option<PlannerGraphEpisode>,
    pub trace_artifact_ref: Option<String>,
}

/// Evidence prepared for a synthesizer lane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceBundle {
    pub summary: String,
    pub items: Vec<EvidenceItem>,
    pub warnings: Vec<String>,
    pub planner: Option<PlannerTraceMetadata>,
}

impl EvidenceBundle {
    pub fn new(summary: impl Into<String>, mut items: Vec<EvidenceItem>) -> Self {
        items.sort_by_key(|item| item.rank);
        Self {
            summary: summary.into(),
            items,
            warnings: Vec::new(),
            planner: None,
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }

    pub fn with_planner(mut self, planner: PlannerTraceMetadata) -> Self {
        self.planner = Some(planner);
        self
    }
}

/// Result returned by a context-gathering lane.
///
/// The gatherer returns evidence for a downstream synthesizer. It does not
/// pretend to be the final answering model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextGatherResult {
    pub capability: GathererCapability,
    pub evidence_bundle: Option<EvidenceBundle>,
}

impl ContextGatherResult {
    pub fn available(evidence_bundle: EvidenceBundle) -> Self {
        Self {
            capability: GathererCapability::Available,
            evidence_bundle: Some(evidence_bundle),
        }
    }

    pub fn unsupported(reason: impl Into<String>) -> Self {
        Self {
            capability: GathererCapability::Unsupported {
                reason: reason.into(),
            },
            evidence_bundle: None,
        }
    }

    pub fn harness_required(reason: impl Into<String>) -> Self {
        Self {
            capability: GathererCapability::HarnessRequired {
                reason: reason.into(),
            },
            evidence_bundle: None,
        }
    }

    pub fn is_synthesis_ready(&self) -> bool {
        matches!(self.capability, GathererCapability::Available) && self.evidence_bundle.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ContextGatherRequest, ContextGatherResult, EvidenceBudget, EvidenceBundle, EvidenceItem,
        GathererCapability, PlannerConfig, PlannerDecision, PlannerStrategyKind,
        PlannerTraceMetadata, PlannerTraceStep, RetainedEvidence, RetrievalMode, RetrievalStrategy,
        RetrieverOption,
    };
    use std::path::PathBuf;

    #[test]
    fn gather_requests_default_to_heuristic_planning() {
        let request = ContextGatherRequest::new(
            "Summarize the runtime path",
            PathBuf::from("."),
            "test",
            EvidenceBudget::default(),
        );

        assert_eq!(request.planning, PlannerConfig::default());
        assert!(request.prior_context.is_empty());
    }

    #[test]
    fn planner_config_can_switch_to_graph_mode_without_changing_strategy() {
        let config = PlannerConfig::default().with_mode(RetrievalMode::Graph);

        assert_eq!(config.mode, RetrievalMode::Graph);
        assert_eq!(config.planner_strategy, PlannerStrategyKind::Heuristic);
        assert_eq!(config.retrieval_strategy, RetrievalStrategy::Lexical);
        assert!(config.retrievers.is_empty());
    }

    #[test]
    fn planner_config_can_capture_structural_retriever_overrides() {
        let config = PlannerConfig::default().with_retrievers(vec![
            RetrieverOption::PathFuzzy,
            RetrieverOption::SegmentFuzzy,
        ]);

        assert_eq!(
            config.retrievers,
            vec![RetrieverOption::PathFuzzy, RetrieverOption::SegmentFuzzy]
        );
    }

    #[test]
    fn available_results_sort_ranked_evidence_and_are_synthesis_ready() {
        let result = ContextGatherResult::available(
            EvidenceBundle::new(
                "Summarized repository evidence",
                vec![
                    EvidenceItem {
                        source: "ARCHITECTURE.md".into(),
                        snippet: "Context gatherers return ranked evidence.".into(),
                        rationale: "Defines the lane split.".into(),
                        rank: 2,
                    },
                    EvidenceItem {
                        source: "src/domain/ports/context_gathering.rs".into(),
                        snippet: "ContextGatherResult wraps evidence and capability.".into(),
                        rationale: "Defines the contract.".into(),
                        rank: 1,
                    },
                ],
            )
            .with_planner(PlannerTraceMetadata {
                mode: RetrievalMode::Linear,
                strategy: PlannerStrategyKind::Heuristic,
                profile: None,
                session_id: Some("session-1".into()),
                completed: true,
                stop_reason: Some("goal-satisfied".into()),
                turn_count: 2,
                steps: vec![PlannerTraceStep {
                    step_id: "step-1".into(),
                    sequence: 1,
                    parent_step_id: None,
                    decisions: vec![PlannerDecision {
                        action: "search".into(),
                        query: Some("runtime lane architecture".into()),
                        rationale: Some("start with the subsystem name".into()),
                        next_step_id: Some("step-2".into()),
                        turn_id: Some("turn-1".into()),
                        branch_id: None,
                        node_id: None,
                        target_branch_id: None,
                        target_node_id: None,
                        edge_id: None,
                        edge_kind: None,
                        frontier_id: None,
                        stop_reason: None,
                    }],
                }],
                retained_artifacts: vec![RetainedEvidence {
                    source: "src/application/mod.rs".into(),
                    snippet: Some(
                        "PreparedRuntimeLanes keeps synthesizer and gatherer lanes.".into(),
                    ),
                    rationale: Some("Carry the runtime lane wiring into the next step.".into()),
                    locator: None,
                }],
                graph_episode: None,
                trace_artifact_ref: None,
            }),
        );

        assert!(result.is_synthesis_ready());
        let bundle = result.evidence_bundle.expect("bundle");
        assert_eq!(bundle.items[0].rank, 1);
        assert_eq!(bundle.items[1].rank, 2);
        let planner = bundle.planner.expect("planner");
        assert_eq!(planner.mode, RetrievalMode::Linear);
        assert_eq!(planner.strategy, PlannerStrategyKind::Heuristic);
        assert_eq!(planner.turn_count, 2);
        assert_eq!(planner.steps[0].step_id, "step-1");
        assert_eq!(
            planner.retained_artifacts[0].source,
            "src/application/mod.rs"
        );
    }

    #[test]
    fn unsupported_results_preserve_reason_without_evidence() {
        let result = ContextGatherResult::unsupported("provider not configured");

        assert!(!result.is_synthesis_ready());
        assert!(matches!(
            result.capability,
            GathererCapability::Unsupported { ref reason }
                if reason == "provider not configured"
        ));
        assert!(result.evidence_bundle.is_none());
    }

    #[test]
    fn harness_required_results_do_not_pretend_to_be_answers() {
        let result =
            ContextGatherResult::harness_required("context-1 requires a dedicated search harness");

        assert!(!result.is_synthesis_ready());
        assert!(matches!(
            result.capability,
            GathererCapability::HarnessRequired { ref reason }
                if reason == "context-1 requires a dedicated search harness"
        ));
        assert!(result.evidence_bundle.is_none());
    }
}
