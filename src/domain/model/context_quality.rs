use serde::{Deserialize, Serialize};

/// Qualitative measure of context degradation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PressureLevel {
    /// No significant truncation detected.
    Low,
    /// Minor truncation (e.g. one factor present).
    Medium,
    /// Significant truncation (e.g. multiple factors or critical component truncation).
    High,
    /// Extreme truncation (e.g. many factors or severe loss of recent context).
    Critical,
}

impl PressureLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// Specific source of context pressure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PressureFactor {
    /// Operator memory (AGENTS.md) was truncated at the character limit.
    MemoryTruncated,
    /// One or more trace artifacts were truncated.
    ArtifactTruncated,
    /// A thread summary was trimmed to its prefix.
    ThreadSummaryTrimmed,
    /// Evidence gathering exceeded the configured budget.
    EvidenceBudgetExhausted,
}

impl PressureFactor {
    pub fn label(&self) -> &'static str {
        match self {
            Self::MemoryTruncated => "memory-truncated",
            Self::ArtifactTruncated => "artifact-truncated",
            Self::ThreadSummaryTrimmed => "thread-summary-trimmed",
            Self::EvidenceBudgetExhausted => "evidence-budget-exhausted",
        }
    }
}

/// Aggregated signal of context quality and pressure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPressure {
    /// Overall pressure level.
    pub level: PressureLevel,
    /// Total number of individual truncation events.
    pub truncation_count: usize,
    /// Set of unique contributing factors.
    pub factors: Vec<PressureFactor>,
}

impl ContextPressure {
    pub fn new(factors: Vec<PressureFactor>, truncation_count: usize) -> Self {
        // FR-04: 0 factors=Low, 1-2=Medium, 3-5=High, 6+=Critical
        // Note: truncation_count might be higher than factors.len() if multiple artifacts are truncated.
        let unique_factors = factors.len();

        let level = if truncation_count == 0 && unique_factors == 0 {
            PressureLevel::Low
        } else if truncation_count <= 2 {
            PressureLevel::Medium
        } else if truncation_count <= 5 {
            PressureLevel::High
        } else {
            PressureLevel::Critical
        };

        Self {
            level,
            truncation_count,
            factors,
        }
    }

    pub fn is_nominal(&self) -> bool {
        self.level == PressureLevel::Low
    }
}

/// Accumulator for context pressure factors during a turn.
#[derive(Clone, Debug, Default)]
pub struct PressureTracker {
    factors: std::collections::HashSet<PressureFactor>,
    truncation_count: usize,
}

impl PressureTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, factor: PressureFactor) {
        self.factors.insert(factor);
        self.truncation_count += 1;
    }

    pub fn record_many(&mut self, factor: PressureFactor, count: usize) {
        if count > 0 {
            self.factors.insert(factor);
            self.truncation_count += count;
        }
    }

    pub fn finish(self) -> ContextPressure {
        let mut factors: Vec<_> = self.factors.into_iter().collect();
        factors.sort_by_key(|f| f.label());
        ContextPressure::new(factors, self.truncation_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_correct_pressure_level() {
        assert_eq!(ContextPressure::new(vec![], 0).level, PressureLevel::Low);

        assert_eq!(
            ContextPressure::new(vec![PressureFactor::MemoryTruncated], 1).level,
            PressureLevel::Medium
        );

        assert_eq!(
            ContextPressure::new(
                vec![
                    PressureFactor::MemoryTruncated,
                    PressureFactor::ArtifactTruncated
                ],
                2
            )
            .level,
            PressureLevel::Medium
        );

        assert_eq!(
            ContextPressure::new(vec![PressureFactor::ArtifactTruncated], 3).level,
            PressureLevel::High
        );

        assert_eq!(
            ContextPressure::new(vec![PressureFactor::ArtifactTruncated], 6).level,
            PressureLevel::Critical
        );
    }

    #[test]
    fn serializes_round_trip() {
        let pressure = ContextPressure::new(
            vec![
                PressureFactor::MemoryTruncated,
                PressureFactor::ArtifactTruncated,
            ],
            2,
        );
        let serialized = serde_json::to_string(&pressure).expect("serialize");
        let deserialized: ContextPressure = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(pressure, deserialized);
    }

    #[test]
    fn pressure_tracker_accumulates_and_finishes() {
        let mut tracker = PressureTracker::new();
        tracker.record(PressureFactor::MemoryTruncated);
        tracker.record_many(PressureFactor::ArtifactTruncated, 2);

        let pressure = tracker.finish();
        assert_eq!(pressure.truncation_count, 3);
        assert_eq!(pressure.factors.len(), 2);
        assert_eq!(pressure.level, PressureLevel::High);
    }
}
