use async_trait::async_trait;
use std::path::PathBuf;

/// Port for specialized context-gathering subagents.
#[async_trait]
pub trait ContextGatherer: Send + Sync {
    /// Report whether this gatherer is actually usable in the current runtime.
    fn capability(&self) -> GathererCapability;

    /// Gather ranked evidence for a retrieval-heavy request.
    async fn gather_context(
        &self,
        request: &ContextGatherRequest,
    ) -> Result<ContextGatherResult, anyhow::Error>;
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
    pub strategy: PlannerStrategyKind,
    pub profile: Option<String>,
    pub step_limit: usize,
    pub retained_artifact_limit: usize,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            strategy: PlannerStrategyKind::Heuristic,
            profile: None,
            step_limit: 3,
            retained_artifact_limit: 5,
        }
    }
}

impl PlannerConfig {
    pub fn with_strategy(mut self, strategy: PlannerStrategyKind) -> Self {
        self.strategy = strategy;
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

/// Capability state for the active gatherer implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GathererCapability {
    Available,
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetainedEvidence {
    pub source: String,
    pub snippet: Option<String>,
    pub rationale: Option<String>,
}

/// Planner metadata returned alongside synthesis-ready evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerTraceMetadata {
    pub strategy: PlannerStrategyKind,
    pub profile: Option<String>,
    pub completed: bool,
    pub stop_reason: Option<String>,
    pub turn_count: usize,
    pub steps: Vec<PlannerTraceStep>,
    pub retained_artifacts: Vec<RetainedEvidence>,
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
        PlannerTraceMetadata, PlannerTraceStep, RetainedEvidence,
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
                strategy: PlannerStrategyKind::Heuristic,
                profile: None,
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
                        stop_reason: None,
                    }],
                }],
                retained_artifacts: vec![RetainedEvidence {
                    source: "src/application/mod.rs".into(),
                    snippet: Some(
                        "PreparedRuntimeLanes keeps synthesizer and gatherer lanes.".into(),
                    ),
                    rationale: Some("Carry the runtime lane wiring into the next step.".into()),
                }],
            }),
        );

        assert!(result.is_synthesis_ready());
        let bundle = result.evidence_bundle.expect("bundle");
        assert_eq!(bundle.items[0].rank, 1);
        assert_eq!(bundle.items[1].rank, 2);
        let planner = bundle.planner.expect("planner");
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
