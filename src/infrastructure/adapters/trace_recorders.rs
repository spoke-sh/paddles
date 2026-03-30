use crate::domain::model::{
    TaskTraceId, TraceBranch, TraceBranchId, TraceCheckpointId, TraceRecord, TraceRecordId,
    TraceRecordKind, TraceReplay,
};
use crate::domain::ports::{TraceRecorder, TraceRecorderCapability};
use anyhow::{Context, Result, anyhow, ensure};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::Mutex;
use transit_core::engine::{LocalEngine, LocalEngineConfig};
use transit_core::kernel::{LineageMetadata, StreamId, StreamPosition};
use transit_core::storage::LineageCheckpoint;

#[derive(Default)]
pub struct InMemoryTraceRecorder {
    records: Mutex<HashMap<TaskTraceId, Vec<TraceRecord>>>,
}

impl InMemoryTraceRecorder {
    pub fn len_for_task(&self, task_id: &TaskTraceId) -> usize {
        self.records
            .lock()
            .expect("in-memory trace recorder lock")
            .get(task_id)
            .map(Vec::len)
            .unwrap_or(0)
    }

    pub fn task_ids(&self) -> Vec<TaskTraceId> {
        self.records
            .lock()
            .expect("in-memory trace recorder lock")
            .keys()
            .cloned()
            .collect()
    }
}

impl TraceRecorder for InMemoryTraceRecorder {
    fn capability(&self) -> TraceRecorderCapability {
        TraceRecorderCapability::Available
    }

    fn record(&self, record: TraceRecord) -> Result<()> {
        let mut guard = self.records.lock().expect("in-memory trace recorder lock");
        guard
            .entry(record.lineage.task_id.clone())
            .or_default()
            .push(record);
        Ok(())
    }

    fn task_ids(&self) -> Vec<TaskTraceId> {
        self.records
            .lock()
            .expect("in-memory trace recorder lock")
            .keys()
            .cloned()
            .collect()
    }

    fn replay(&self, task_id: &TaskTraceId) -> Result<TraceReplay> {
        let mut records = self
            .records
            .lock()
            .expect("in-memory trace recorder lock")
            .get(task_id)
            .cloned()
            .unwrap_or_default();
        records.sort_by_key(|record| record.sequence);
        Ok(TraceReplay {
            task_id: task_id.clone(),
            records,
        })
    }
}

pub struct TransitTraceRecorder {
    engine: LocalEngine,
    state: Mutex<TransitRecorderState>,
}

#[derive(Default)]
struct TransitRecorderState {
    tasks: HashMap<TaskTraceId, TransitTaskState>,
}

struct TransitTaskState {
    root_stream: StreamId,
    branch_streams: HashMap<TraceBranchId, StreamId>,
    record_positions: HashMap<TraceRecordId, StreamPosition>,
    checkpoints: HashMap<TraceCheckpointId, LineageCheckpoint>,
}

impl TransitTraceRecorder {
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self> {
        let engine = LocalEngine::open(LocalEngineConfig::new(data_dir.as_ref()))
            .context("open embedded transit engine")?;
        Ok(Self {
            engine,
            state: Mutex::new(TransitRecorderState::default()),
        })
    }

    pub fn verify_checkpoints(&self, task_id: &TaskTraceId) -> Result<()> {
        let checkpoints = {
            let guard = self.state.lock().expect("transit trace recorder lock");
            guard
                .tasks
                .get(task_id)
                .map(|task| task.checkpoints.values().cloned().collect::<Vec<_>>())
                .unwrap_or_default()
        };

        for checkpoint in checkpoints {
            self.engine
                .verify_checkpoint(&checkpoint)
                .with_context(|| format!("verify checkpoint {}", checkpoint.kind))?;
        }

        Ok(())
    }

    pub fn stream_count(&self, task_id: &TaskTraceId) -> usize {
        self.state
            .lock()
            .expect("transit trace recorder lock")
            .tasks
            .get(task_id)
            .map(|task| 1 + task.branch_streams.len())
            .unwrap_or(0)
    }

    fn create_task_root_if_needed(&self, record: &TraceRecord) -> Result<StreamId> {
        let mut guard = self.state.lock().expect("transit trace recorder lock");
        if let Some(existing) = guard.tasks.get(&record.lineage.task_id) {
            return Ok(existing.root_stream.clone());
        }

        ensure!(
            matches!(record.kind, TraceRecordKind::TaskRootStarted(_)),
            "embedded transit recording requires a task root record before any other record"
        );

        let root_stream = StreamId::new(format!(
            "paddles.task.{}.root",
            record.lineage.task_id.as_str()
        ))?;
        self.engine
            .create_stream(transit_core::kernel::StreamDescriptor::root(
                root_stream.clone(),
                LineageMetadata::new(Some("paddles".into()), Some("task-root".into()))
                    .with_label("task_id", record.lineage.task_id.as_str()),
            ))?;
        guard.tasks.insert(
            record.lineage.task_id.clone(),
            TransitTaskState {
                root_stream: root_stream.clone(),
                branch_streams: HashMap::new(),
                record_positions: HashMap::new(),
                checkpoints: HashMap::new(),
            },
        );
        Ok(root_stream)
    }

    fn target_stream_for_record(
        &self,
        record: &TraceRecord,
        task: &TransitTaskState,
    ) -> Result<StreamId> {
        match &record.kind {
            TraceRecordKind::PlannerBranchDeclared(branch) => Ok(task
                .branch_streams
                .get(&branch.branch_id)
                .cloned()
                .unwrap_or_else(|| task.root_stream.clone())),
            _ => record
                .lineage
                .branch_id
                .as_ref()
                .map(|branch_id| {
                    task.branch_streams
                        .get(branch_id)
                        .cloned()
                        .ok_or_else(|| anyhow!("unknown branch stream '{}'", branch_id.as_str()))
                })
                .transpose()?
                .map(Ok)
                .unwrap_or_else(|| Ok(task.root_stream.clone())),
        }
    }

    fn ensure_branch_stream(&self, record: &TraceRecord, branch: &TraceBranch) -> Result<()> {
        let mut guard = self.state.lock().expect("transit trace recorder lock");
        let task = guard
            .tasks
            .get_mut(&record.lineage.task_id)
            .ok_or_else(|| {
                anyhow!(
                    "missing task root for branch '{}'",
                    branch.branch_id.as_str()
                )
            })?;
        if task.branch_streams.contains_key(&branch.branch_id) {
            return Ok(());
        }

        let parent_record_id = record.lineage.parent_record_id.as_ref().ok_or_else(|| {
            anyhow!(
                "branch '{}' is missing a parent record id",
                branch.branch_id.as_str()
            )
        })?;
        let parent_position = task
            .record_positions
            .get(parent_record_id)
            .cloned()
            .ok_or_else(|| anyhow!("unknown parent record '{}'", parent_record_id.as_str()))?;
        let branch_stream = StreamId::new(format!(
            "paddles.task.{}.branch.{}",
            record.lineage.task_id.as_str(),
            branch.branch_id.as_str()
        ))?;
        self.engine.create_branch(
            branch_stream.clone(),
            parent_position,
            LineageMetadata::new(Some("paddles".into()), Some("planner-branch".into()))
                .with_branch_kind("conversation-thread")
                .with_anchor_ref(parent_record_id.as_str())
                .with_label("task_id", record.lineage.task_id.as_str())
                .with_label("branch_id", branch.branch_id.as_str())
                .with_label("label", branch.label.clone()),
        )?;
        task.branch_streams
            .insert(branch.branch_id.clone(), branch_stream);
        Ok(())
    }
}

impl TraceRecorder for TransitTraceRecorder {
    fn capability(&self) -> TraceRecorderCapability {
        TraceRecorderCapability::Available
    }

    fn record(&self, record: TraceRecord) -> Result<()> {
        self.create_task_root_if_needed(&record)?;

        if let TraceRecordKind::PlannerBranchDeclared(branch) = &record.kind {
            self.ensure_branch_stream(&record, branch)?;
        }

        let stream_id = {
            let guard = self.state.lock().expect("transit trace recorder lock");
            let task = guard.tasks.get(&record.lineage.task_id).ok_or_else(|| {
                anyhow!(
                    "missing task state for '{}'",
                    record.lineage.task_id.as_str()
                )
            })?;
            self.target_stream_for_record(&record, task)?
        };

        let encoded = serde_json::to_vec(&record).context("serialize trace record")?;
        let outcome = self.engine.append(&stream_id, encoded)?;

        let mut guard = self.state.lock().expect("transit trace recorder lock");
        let task = guard
            .tasks
            .get_mut(&record.lineage.task_id)
            .ok_or_else(|| {
                anyhow!(
                    "missing task state for '{}'",
                    record.lineage.task_id.as_str()
                )
            })?;
        task.record_positions
            .insert(record.record_id.clone(), outcome.position().clone());

        if let TraceRecordKind::CompletionCheckpoint(checkpoint) = &record.kind {
            let receipt = self
                .engine
                .checkpoint(&stream_id, checkpoint.kind.label())
                .with_context(|| format!("checkpoint stream '{}'", stream_id.as_str()))?;
            task.checkpoints
                .insert(checkpoint.checkpoint_id.clone(), receipt);
        }

        Ok(())
    }

    fn replay(&self, task_id: &TaskTraceId) -> Result<TraceReplay> {
        let streams = {
            let guard = self.state.lock().expect("transit trace recorder lock");
            let Some(task) = guard.tasks.get(task_id) else {
                return Ok(TraceReplay {
                    task_id: task_id.clone(),
                    records: Vec::new(),
                });
            };
            let mut streams = vec![task.root_stream.clone()];
            streams.extend(task.branch_streams.values().cloned());
            streams
        };

        let mut records = BTreeMap::<(u64, String), TraceRecord>::new();
        for stream_id in streams {
            for local_record in self.engine.replay(&stream_id)? {
                let record: TraceRecord = serde_json::from_slice(local_record.payload())
                    .context("deserialize transit trace record")?;
                records.insert(
                    (record.sequence, record.record_id.as_str().to_string()),
                    record,
                );
            }
        }

        Ok(TraceReplay {
            task_id: task_id.clone(),
            records: records.into_values().collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{InMemoryTraceRecorder, TransitTraceRecorder};
    use crate::domain::model::{
        ArtifactEnvelope, ArtifactKind, TaskTraceId, TraceArtifactId, TraceCheckpointId,
        TraceCheckpointKind, TraceCompletionCheckpoint, TraceLineage, TraceRecord, TraceRecordId,
        TraceRecordKind, TraceTaskRoot, TurnTraceId,
    };
    use crate::domain::ports::TraceRecorder;
    use tempfile::tempdir;

    fn root_record(task: &str) -> TraceRecord {
        TraceRecord {
            record_id: TraceRecordId::new("record-1").expect("record id"),
            sequence: 1,
            lineage: TraceLineage {
                task_id: TaskTraceId::new(task).expect("task id"),
                turn_id: TurnTraceId::new("turn-1").expect("turn id"),
                branch_id: None,
                parent_record_id: None,
            },
            kind: TraceRecordKind::TaskRootStarted(TraceTaskRoot {
                prompt: ArtifactEnvelope::text(
                    TraceArtifactId::new("artifact-1").expect("artifact"),
                    ArtifactKind::Prompt,
                    "prompt",
                    "hello",
                    256,
                ),
                interpretation: None,
                planner_model: "qwen-1.5b".to_string(),
                synthesizer_model: "qwen-1.5b".to_string(),
            }),
        }
    }

    #[test]
    fn in_memory_recorder_replays_records_in_sequence_order() {
        let recorder = InMemoryTraceRecorder::default();
        let task_id = TaskTraceId::new("task-1").expect("task id");
        recorder.record(root_record("task-1")).expect("record root");

        let replay = recorder.replay(&task_id).expect("replay");
        assert_eq!(replay.records.len(), 1);
        assert_eq!(recorder.len_for_task(&task_id), 1);
    }

    #[test]
    fn transit_recorder_replays_root_and_verifies_checkpoint() {
        let temp = tempdir().expect("tempdir");
        let recorder = TransitTraceRecorder::open(temp.path()).expect("transit recorder");
        let mut root = root_record("task-2");
        let task_id = root.lineage.task_id.clone();
        recorder.record(root.clone()).expect("record root");

        root.record_id = TraceRecordId::new("record-2").expect("record id");
        root.sequence = 2;
        root.lineage.parent_record_id = Some(TraceRecordId::new("record-1").expect("record id"));
        root.kind = TraceRecordKind::CompletionCheckpoint(TraceCompletionCheckpoint {
            checkpoint_id: TraceCheckpointId::new("checkpoint-1").expect("checkpoint id"),
            kind: TraceCheckpointKind::TurnCompleted,
            summary: "turn completed".to_string(),
            response: None,
            citations: Vec::new(),
            grounded: false,
        });
        recorder.record(root).expect("record checkpoint");

        let replay = recorder.replay(&task_id).expect("replay");
        assert_eq!(replay.records.len(), 2);
        assert_eq!(recorder.stream_count(&task_id), 1);
        recorder
            .verify_checkpoints(&task_id)
            .expect("verify checkpoints");
    }
}
