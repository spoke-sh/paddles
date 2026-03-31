use crate::domain::model::traces::TraceRecordKind;
use crate::domain::ports::{ContextResolver, TraceRecorder};
use crate::infrastructure::adapters::trace_recorders::TransitTraceRecorder;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use paddles_conversation::ContextLocator;
use std::sync::Arc;

#[derive(Debug)]
pub struct TransitContextResolver {
    recorder: Arc<TransitTraceRecorder>,
}

impl TransitContextResolver {
    pub fn new(recorder: Arc<TransitTraceRecorder>) -> Self {
        Self { recorder }
    }
}

#[async_trait]
impl ContextResolver for TransitContextResolver {
    async fn resolve(&self, locator: &ContextLocator) -> Result<String> {
        match locator {
            ContextLocator::Inline { content } => Ok(content.clone()),
            ContextLocator::Transit { task_id, record_id } => {
                // Replay the task to find the specific record.
                // Note: This is a synchronous operation in TransitTraceRecorder today.
                let replay = self.recorder.replay(task_id)?;
                let record = replay
                    .records
                    .into_iter()
                    .find(|r| r.record_id == *record_id)
                    .ok_or_else(|| {
                        anyhow!("record not found in transit: {}", record_id.as_str())
                    })?;

                // Extract content from the record kind.
                match record.kind {
                    TraceRecordKind::TaskRootStarted(root) => {
                        Ok(root.prompt.inline_content.unwrap_or(root.prompt.summary))
                    }
                    TraceRecordKind::TurnStarted(started) => Ok(started
                        .prompt
                        .inline_content
                        .unwrap_or(started.prompt.summary)),
                    TraceRecordKind::SelectionArtifact(selection) => Ok(selection
                        .artifact
                        .inline_content
                        .unwrap_or(selection.artifact.summary)),
                    TraceRecordKind::ToolCallRequested(call) => {
                        Ok(call.payload.inline_content.unwrap_or(call.payload.summary))
                    }
                    TraceRecordKind::ToolCallCompleted(call) => {
                        Ok(call.payload.inline_content.unwrap_or(call.payload.summary))
                    }
                    TraceRecordKind::CompletionCheckpoint(checkpoint) => {
                        if let Some(resp) = checkpoint.response {
                            Ok(resp.inline_content.unwrap_or(resp.summary))
                        } else {
                            Ok(checkpoint.summary)
                        }
                    }
                    _ => Ok(format!("{:?}", record.kind)),
                }
            }
            ContextLocator::Filesystem { path } => {
                let content = tokio::fs::read_to_string(path).await?;
                Ok(content)
            }
            ContextLocator::Sift { index_ref } => Err(anyhow!(
                "Sift resolution not yet implemented: {}",
                index_ref
            )),
        }
    }
}

#[derive(Debug)]
pub struct NoopContextResolver;

#[async_trait]
impl ContextResolver for NoopContextResolver {
    async fn resolve(&self, _locator: &ContextLocator) -> Result<String> {
        Err(anyhow!("Context resolution not available in this runtime."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::traces::{TraceLineage, TraceRecord, TraceRecordKind, TraceTaskRoot};
    use crate::domain::model::{ArtifactEnvelope, ArtifactKind};
    use crate::domain::ports::TraceRecorder;
    use paddles_conversation::{TaskTraceId, TraceArtifactId, TraceRecordId, TurnTraceId};
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn resolves_transit_locator() {
        let temp = tempdir().expect("tempdir");
        let recorder = Arc::new(TransitTraceRecorder::open(temp.path()).expect("transit recorder"));
        let resolver = TransitContextResolver::new(recorder.clone());

        let task_id = TaskTraceId::new("task-1").expect("task id");
        let record_id = TraceRecordId::new("record-1").expect("record id");

        let record = TraceRecord {
            record_id: record_id.clone(),
            sequence: 1,
            lineage: TraceLineage {
                task_id: task_id.clone(),
                turn_id: TurnTraceId::new("turn-1").expect("turn id"),
                branch_id: None,
                parent_record_id: None,
            },
            kind: TraceRecordKind::TaskRootStarted(TraceTaskRoot {
                prompt: ArtifactEnvelope::text(
                    TraceArtifactId::new("artifact-1").expect("artifact"),
                    ArtifactKind::Prompt,
                    "prompt",
                    "hello world",
                    256,
                ),
                interpretation: None,
                planner_model: "qwen-1.5b".to_string(),
                synthesizer_model: "qwen-1.5b".to_string(),
            }),
        };

        recorder.record(record).expect("record");

        let locator = ContextLocator::Transit {
            task_id: task_id.clone(),
            record_id: record_id.clone(),
        };

        let resolved = resolver.resolve(&locator).await.expect("resolve");
        assert_eq!(resolved, "hello world");
    }

    #[tokio::test]
    async fn fails_closed_for_missing_transit_record() {
        let temp = tempdir().expect("tempdir");
        let recorder = Arc::new(TransitTraceRecorder::open(temp.path()).expect("transit recorder"));
        let resolver = TransitContextResolver::new(recorder);

        let locator = ContextLocator::Transit {
            task_id: TaskTraceId::new("nonexistent-task").expect("task id"),
            record_id: TraceRecordId::new("nonexistent-record").expect("record id"),
        };

        let result = resolver.resolve(&locator).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn fails_closed_for_sift_tier() {
        let temp = tempdir().expect("tempdir");
        let recorder = Arc::new(TransitTraceRecorder::open(temp.path()).expect("transit recorder"));
        let resolver = TransitContextResolver::new(recorder);

        let locator = ContextLocator::Sift {
            index_ref: "some-index".to_string(),
        };

        let result = resolver.resolve(&locator).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not yet implemented")
        );
    }

    #[tokio::test]
    async fn noop_resolver_fails_closed() {
        let resolver = NoopContextResolver;
        let locator = ContextLocator::Inline {
            content: "test".to_string(),
        };
        let result = resolver.resolve(&locator).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn resolves_filesystem_locator() {
        let temp = tempdir().expect("tempdir");
        let recorder = Arc::new(TransitTraceRecorder::open(temp.path()).expect("transit recorder"));
        let resolver = TransitContextResolver::new(recorder);

        let file_path = temp.path().join("test.txt");
        tokio::fs::write(&file_path, "file content")
            .await
            .expect("write");

        let locator = ContextLocator::Filesystem { path: file_path };

        let resolved = resolver.resolve(&locator).await.expect("resolve");
        assert_eq!(resolved, "file content");
    }
}
