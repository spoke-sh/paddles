use paddles_conversation::TraceArtifactId;
use serde::{Deserialize, Serialize};

/// Size limits for context compaction self-assessment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionBudget {
    /// Maximum number of planner steps allowed for self-assessment.
    pub max_steps: usize,
}

impl Default for CompactionBudget {
    fn default() -> Self {
        Self { max_steps: 3 }
    }
}

/// Request for context compaction self-assessment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionRequest {
    /// The artifacts to be evaluated for compaction.
    pub target_scope: Vec<TraceArtifactId>,
    /// A high-level summary of the current goal/context to guide relevance assessment.
    pub relevance_query: String,
    /// Constraints for the compaction process.
    pub budget: CompactionBudget,
}

/// A single decision made by the self-assessment engine for a context artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompactionDecision {
    /// Keep the artifact as-is with a given priority.
    Keep { priority: usize },
    /// Replace the artifact with a new summary.
    Compact { summary: String },
    /// Discard the artifact as irrelevant.
    Discard { reason: String },
}

/// The result of a context compaction self-assessment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionPlan {
    /// Decisions mapped by artifact ID.
    pub decisions: std::collections::HashMap<TraceArtifactId, CompactionDecision>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compaction_request_serializes_round_trip() {
        let artifact_id = TraceArtifactId::new("art-1").unwrap();
        let request = CompactionRequest {
            target_scope: vec![artifact_id.clone()],
            relevance_query: "find the bug".to_string(),
            budget: CompactionBudget::default(),
        };

        let serialized = serde_json::to_string(&request).expect("serialize");
        let deserialized: CompactionRequest =
            serde_json::from_str(&serialized).expect("deserialize");

        assert_eq!(request, deserialized);
    }

    #[test]
    fn compaction_plan_serializes_round_trip() {
        let artifact_id = TraceArtifactId::new("art-1").unwrap();
        let mut decisions = std::collections::HashMap::new();
        decisions.insert(artifact_id, CompactionDecision::Keep { priority: 1 });

        let plan = CompactionPlan { decisions };

        let serialized = serde_json::to_string(&plan).expect("serialize");
        let deserialized: CompactionPlan = serde_json::from_str(&serialized).expect("deserialize");

        assert_eq!(plan, deserialized);
    }
}
