use serde::{Deserialize, Serialize};

/// Qualitative measure of context degradation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StrainLevel {
    /// No significant truncation detected.
    Low,
    /// Minor truncation (e.g. one factor present).
    Medium,
    /// Significant truncation (e.g. multiple factors or critical component truncation).
    High,
    /// Extreme truncation (e.g. many factors or severe loss of recent context).
    Critical,
}

impl StrainLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// Specific source of context strain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrainFactor {
    /// Operator memory (AGENTS.md) was truncated at the character limit.
    MemoryTruncated,
    /// One or more trace artifacts were truncated.
    ArtifactTruncated,
    /// A thread summary was trimmed to its prefix.
    ThreadSummaryTrimmed,
    /// Evidence gathering exceeded the configured budget.
    EvidenceBudgetExhausted,
}

impl StrainFactor {
    pub fn label(&self) -> &'static str {
        match self {
            Self::MemoryTruncated => "memory-truncated",
            Self::ArtifactTruncated => "artifact-truncated",
            Self::ThreadSummaryTrimmed => "thread-summary-trimmed",
            Self::EvidenceBudgetExhausted => "evidence-budget-exhausted",
        }
    }
}

/// Aggregated signal of context quality and strain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextStrain {
    /// Overall strain level.
    pub level: StrainLevel,
    /// Total number of individual truncation events.
    pub truncation_count: usize,
    /// Set of unique contributing factors.
    pub factors: Vec<StrainFactor>,
}

impl ContextStrain {
    pub fn new(factors: Vec<StrainFactor>, truncation_count: usize) -> Self {
        // FR-04: 0 factors=Low, 1-2=Medium, 3-5=High, 6+=Critical
        // Note: truncation_count might be higher than factors.len() if multiple artifacts are truncated.
        let unique_factors = factors.len();

        let level = if truncation_count == 0 && unique_factors == 0 {
            StrainLevel::Low
        } else if truncation_count <= 2 {
            StrainLevel::Medium
        } else if truncation_count <= 5 {
            StrainLevel::High
        } else {
            StrainLevel::Critical
        };

        Self {
            level,
            truncation_count,
            factors,
        }
    }

    pub fn is_nominal(&self) -> bool {
        self.level == StrainLevel::Low
    }
}

/// Accumulator for context strain factors during a turn.
#[derive(Clone, Debug, Default)]
pub struct StrainTracker {
    factors: std::collections::HashSet<StrainFactor>,
    truncation_count: usize,
}

impl StrainTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, factor: StrainFactor) {
        self.factors.insert(factor);
        self.truncation_count += 1;
    }

    pub fn record_many(&mut self, factor: StrainFactor, count: usize) {
        if count > 0 {
            self.factors.insert(factor);
            self.truncation_count += count;
        }
    }

    pub fn finish(self) -> ContextStrain {
        let mut factors: Vec<_> = self.factors.into_iter().collect();
        factors.sort_by_key(|f| f.label());
        ContextStrain::new(factors, self.truncation_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_correct_strain_level() {
        assert_eq!(ContextStrain::new(vec![], 0).level, StrainLevel::Low);

        assert_eq!(
            ContextStrain::new(vec![StrainFactor::MemoryTruncated], 1).level,
            StrainLevel::Medium
        );

        assert_eq!(
            ContextStrain::new(
                vec![
                    StrainFactor::MemoryTruncated,
                    StrainFactor::ArtifactTruncated
                ],
                2
            )
            .level,
            StrainLevel::Medium
        );

        assert_eq!(
            ContextStrain::new(vec![StrainFactor::ArtifactTruncated], 3).level,
            StrainLevel::High
        );

        assert_eq!(
            ContextStrain::new(vec![StrainFactor::ArtifactTruncated], 6).level,
            StrainLevel::Critical
        );
    }

    #[test]
    fn serializes_round_trip() {
        let strain = ContextStrain::new(
            vec![
                StrainFactor::MemoryTruncated,
                StrainFactor::ArtifactTruncated,
            ],
            2,
        );
        let serialized = serde_json::to_string(&strain).expect("serialize");
        let deserialized: ContextStrain = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(strain, deserialized);
    }

    #[test]
    fn strain_tracker_accumulates_and_finishes() {
        let mut tracker = StrainTracker::new();
        tracker.record(StrainFactor::MemoryTruncated);
        tracker.record_many(StrainFactor::ArtifactTruncated, 2);

        let strain = tracker.finish();
        assert_eq!(strain.truncation_count, 3);
        assert_eq!(strain.factors.len(), 2);
        assert_eq!(strain.level, StrainLevel::High);
    }
}
