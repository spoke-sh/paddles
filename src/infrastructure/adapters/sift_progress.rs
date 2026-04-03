use sift::{SearchProgress, SearchTelemetry};
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SiftProgressDisplay {
    pub phase: String,
    pub detail: String,
    pub estimated_remaining: Option<Duration>,
}

pub fn describe_sift_progress(
    progress: &SearchProgress,
    telemetry: &SearchTelemetry,
) -> SiftProgressDisplay {
    match progress {
        SearchProgress::Indexing {
            phase,
            files_processed,
            files_total,
            estimated_remaining,
        } => SiftProgressDisplay {
            phase: phase.to_string(),
            detail: format!(
                "indexing {files_processed}/{files_total} files · blobs {} · fresh {} · skipped {} · segments {} · bm25 cache {} build {}",
                telemetry.blob_hits,
                telemetry.fresh_artifact_builds,
                telemetry.skipped_artifacts,
                telemetry.total_segments,
                telemetry.bm25_index_cache_hits,
                telemetry.bm25_index_builds,
            ),
            estimated_remaining: *estimated_remaining,
        },
        SearchProgress::Embedding {
            phase,
            chunks_processed,
            chunks_total,
            estimated_remaining,
        } => SiftProgressDisplay {
            phase: phase.to_string(),
            detail: format!(
                "embedding {chunks_processed}/{chunks_total} chunks · embed cache {} · segments {}",
                telemetry.embedding_hits, telemetry.total_segments,
            ),
            estimated_remaining: *estimated_remaining,
        },
        SearchProgress::PlannerStep {
            phase,
            step_index,
            action,
            query,
            estimated_remaining,
        } => SiftProgressDisplay {
            phase: phase.to_string(),
            detail: format!(
                "planning step {} · {}{}",
                step_index + 1,
                action,
                query
                    .as_deref()
                    .map(|value| format!(" · query: {value}"))
                    .unwrap_or_default()
            ),
            estimated_remaining: *estimated_remaining,
        },
        SearchProgress::Retrieving {
            phase,
            turn_index,
            turns_total,
            estimated_remaining,
        } => SiftProgressDisplay {
            phase: phase.to_string(),
            detail: format!(
                "retrieving turn {}/{} · bm25 cache {} build {}",
                turn_index + 1,
                turns_total,
                telemetry.bm25_index_cache_hits,
                telemetry.bm25_index_builds,
            ),
            estimated_remaining: *estimated_remaining,
        },
        SearchProgress::Ranking {
            phase,
            results_processed,
            results_total,
            estimated_remaining,
        } => SiftProgressDisplay {
            phase: phase.to_string(),
            detail: format!("ranking {results_processed}/{results_total} hits"),
            estimated_remaining: *estimated_remaining,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::describe_sift_progress;
    use sift::{SearchPhase, SearchProgress, SearchTelemetry};
    use std::time::Duration;

    fn telemetry() -> SearchTelemetry {
        SearchTelemetry {
            heuristic_hits: 3,
            blob_hits: 7,
            fresh_artifact_builds: 2,
            skipped_artifacts: 1,
            embedding_hits: 5,
            total_files: 10,
            total_segments: 24,
            bm25_index_cache_hits: 4,
            bm25_index_builds: 1,
        }
    }

    #[test]
    fn indexing_progress_includes_incremental_reuse_metrics() {
        let display = describe_sift_progress(
            &SearchProgress::Indexing {
                phase: SearchPhase::Indexing,
                files_processed: 4,
                files_total: 10,
                estimated_remaining: Some(Duration::from_secs(12)),
            },
            &telemetry(),
        );

        assert_eq!(display.phase, "Indexing");
        assert!(display.detail.contains("indexing 4/10 files"));
        assert!(display.detail.contains("blobs 7"));
        assert!(display.detail.contains("fresh 2"));
        assert!(display.detail.contains("bm25 cache 4 build 1"));
        assert_eq!(display.estimated_remaining, Some(Duration::from_secs(12)));
    }

    #[test]
    fn embedding_progress_includes_embedding_cache_hits() {
        let display = describe_sift_progress(
            &SearchProgress::Embedding {
                phase: SearchPhase::Embedding,
                chunks_processed: 3,
                chunks_total: 9,
                estimated_remaining: Some(Duration::from_secs(4)),
            },
            &telemetry(),
        );

        assert_eq!(display.phase, "Embedding");
        assert!(display.detail.contains("embedding 3/9 chunks"));
        assert!(display.detail.contains("embed cache 5"));
        assert_eq!(display.estimated_remaining, Some(Duration::from_secs(4)));
    }
}
