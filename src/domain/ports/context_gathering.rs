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
            prior_context: Vec::new(),
        }
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

/// Evidence prepared for a synthesizer lane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceBundle {
    pub summary: String,
    pub items: Vec<EvidenceItem>,
    pub warnings: Vec<String>,
}

impl EvidenceBundle {
    pub fn new(summary: impl Into<String>, mut items: Vec<EvidenceItem>) -> Self {
        items.sort_by_key(|item| item.rank);
        Self {
            summary: summary.into(),
            items,
            warnings: Vec::new(),
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
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
    use super::{ContextGatherResult, EvidenceBundle, EvidenceItem, GathererCapability};

    #[test]
    fn available_results_sort_ranked_evidence_and_are_synthesis_ready() {
        let result = ContextGatherResult::available(EvidenceBundle::new(
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
        ));

        assert!(result.is_synthesis_ready());
        let bundle = result.evidence_bundle.expect("bundle");
        assert_eq!(bundle.items[0].rank, 1);
        assert_eq!(bundle.items[1].rank, 2);
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
