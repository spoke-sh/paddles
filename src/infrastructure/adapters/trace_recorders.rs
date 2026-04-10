use crate::domain::model::{
    TaskTraceId, TraceBranch, TraceBranchId, TraceCheckpointId, TraceRecord, TraceRecordId,
    TraceRecordKind, TraceReplay,
};
use crate::domain::ports::{TraceRecorder, TraceRecorderCapability};
use anyhow::{Context, Result, anyhow, ensure};
use directories::ProjectDirs;
use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use transit_core::engine::{LocalEngine, LocalEngineConfig};
use transit_core::kernel::{LineageMetadata, StreamId, StreamPosition};
use transit_core::storage::LineageCheckpoint;

pub struct InMemoryTraceRecorder {
    records: Mutex<HashMap<TaskTraceId, Vec<TraceRecord>>>,
    capability: TraceRecorderCapability,
}

impl Default for InMemoryTraceRecorder {
    fn default() -> Self {
        Self::new_ephemeral("in-memory session spine does not survive process restarts")
    }
}

impl InMemoryTraceRecorder {
    pub fn new_ephemeral(reason: impl Into<String>) -> Self {
        Self {
            records: Mutex::new(HashMap::new()),
            capability: TraceRecorderCapability::Ephemeral {
                backend: "in_memory".to_string(),
                reason: reason.into(),
            },
        }
    }

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
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn capability(&self) -> TraceRecorderCapability {
        self.capability.clone()
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

#[derive(Clone)]
pub struct TransitTraceRecorder {
    pub(crate) data_dir: PathBuf,
    pub(crate) engine: Arc<LocalEngine>,
    pub(crate) state: Arc<Mutex<TransitRecorderState>>,
}

impl std::fmt::Debug for TransitTraceRecorder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransitTraceRecorder").finish()
    }
}

#[derive(Default)]
pub(crate) struct TransitRecorderState {
    pub(crate) tasks: HashMap<TaskTraceId, TransitTaskState>,
}

pub(crate) struct TransitTaskState {
    pub(crate) root_stream: StreamId,
    pub(crate) branch_streams: HashMap<TraceBranchId, StreamId>,
    pub(crate) record_positions: HashMap<TraceRecordId, StreamPosition>,
    pub(crate) checkpoints: HashMap<TraceCheckpointId, LineageCheckpoint>,
}

impl TransitTraceRecorder {
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self> {
        fs::create_dir_all(data_dir.as_ref()).with_context(|| {
            format!(
                "create recorder state directory {}",
                data_dir.as_ref().display()
            )
        })?;
        let engine = LocalEngine::open(LocalEngineConfig::new(data_dir.as_ref()))
            .context("open embedded transit engine")?;
        Ok(Self {
            data_dir: data_dir.as_ref().to_path_buf(),
            engine: Arc::new(engine),
            state: Arc::new(Mutex::new(TransitRecorderState::default())),
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
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn capability(&self) -> TraceRecorderCapability {
        TraceRecorderCapability::Persistent {
            backend: "embedded_transit".to_string(),
            location: self.data_dir.display().to_string(),
        }
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
                let record: TraceRecord =
                    serde_json::from_slice::<TraceRecord>(local_record.payload())
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

const TRACE_RECORDER_STATE_ROOT_DIR: &str = "trace-sessions";
const TRACE_RECORDER_WORKSPACES_DIR: &str = "workspaces";

pub fn default_trace_recorder_for_workspace(workspace_root: &Path) -> Arc<dyn TraceRecorder> {
    let state_root = default_trace_recorder_state_root();
    default_trace_recorder_for_workspace_under_root(workspace_root, &state_root)
}

pub(crate) fn default_trace_recorder_for_workspace_under_root(
    workspace_root: &Path,
    state_root: &Path,
) -> Arc<dyn TraceRecorder> {
    let workspace_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());
    let data_dir = state_root
        .join(TRACE_RECORDER_WORKSPACES_DIR)
        .join(workspace_cache_leaf(&workspace_root));
    match TransitTraceRecorder::open(&data_dir) {
        Ok(recorder) => Arc::new(recorder),
        Err(error) => Arc::new(InMemoryTraceRecorder::new_ephemeral(format!(
            "embedded transit session spine unavailable at {}: {error}",
            data_dir.display()
        ))),
    }
}

fn default_trace_recorder_state_root() -> PathBuf {
    if let Some(project_dirs) = ProjectDirs::from("", "", "paddles")
        && let Some(state_dir) = project_dirs.state_dir()
    {
        return state_dir.join(TRACE_RECORDER_STATE_ROOT_DIR);
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("paddles")
            .join(TRACE_RECORDER_STATE_ROOT_DIR);
    }

    PathBuf::from(".paddles-state").join(TRACE_RECORDER_STATE_ROOT_DIR)
}

fn workspace_cache_leaf(workspace_root: &Path) -> String {
    let workspace_name = workspace_root
        .file_name()
        .and_then(|segment| segment.to_str())
        .map(sanitize_component)
        .filter(|segment| !segment.is_empty())
        .unwrap_or_else(|| "workspace".to_string());
    format!(
        "{}-{:016x}",
        workspace_name,
        stable_workspace_hash(workspace_root)
    )
}

fn sanitize_component(component: &str) -> String {
    component
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn stable_workspace_hash(workspace_root: &Path) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    workspace_root.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::{
        InMemoryTraceRecorder, TransitTraceRecorder,
        default_trace_recorder_for_workspace_under_root,
    };
    use crate::domain::model::{
        ArtifactEnvelope, ArtifactKind, TaskTraceId, TraceArtifactId, TraceCheckpointId,
        TraceCheckpointKind, TraceCompletionCheckpoint, TraceLineage, TraceRecord, TraceRecordId,
        TraceRecordKind, TraceTaskRoot, TraceTurnStarted, TurnTraceId,
    };
    use crate::domain::ports::{
        TraceRecorder, TraceRecorderCapability, TraceReplaySliceAnchor, TraceReplaySliceRequest,
    };
    use std::fs;
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

    fn turn_started_record(
        task_id: &TaskTraceId,
        turn_id: &TurnTraceId,
        record_id: &str,
        sequence: u64,
        parent_record_id: &str,
        content: &str,
    ) -> TraceRecord {
        TraceRecord {
            record_id: TraceRecordId::new(record_id).expect("record id"),
            sequence,
            lineage: TraceLineage {
                task_id: task_id.clone(),
                turn_id: turn_id.clone(),
                branch_id: None,
                parent_record_id: Some(
                    TraceRecordId::new(parent_record_id).expect("parent record id"),
                ),
            },
            kind: TraceRecordKind::TurnStarted(TraceTurnStarted {
                prompt: ArtifactEnvelope::text(
                    TraceArtifactId::new(format!("artifact-{record_id}")).expect("artifact"),
                    ArtifactKind::Prompt,
                    format!("prompt {record_id}"),
                    content,
                    256,
                ),
                interpretation: None,
                planner_model: "qwen-1.5b".to_string(),
                synthesizer_model: "qwen-1.5b".to_string(),
                thread: crate::domain::model::ConversationThreadRef::Mainline,
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

    #[test]
    fn wake_reports_latest_checkpoint_and_resume_cursor() {
        let recorder = InMemoryTraceRecorder::default();
        let task_id = TaskTraceId::new("task-wake").expect("task id");
        let turn_one = TurnTraceId::new("task-wake.turn-0001").expect("turn");
        let turn_two = TurnTraceId::new("task-wake.turn-0002").expect("turn");
        recorder
            .record(root_record("task-wake"))
            .expect("record root");
        recorder
            .record(TraceRecord {
                record_id: TraceRecordId::new("record-2").expect("record id"),
                sequence: 2,
                lineage: TraceLineage {
                    task_id: task_id.clone(),
                    turn_id: turn_one,
                    branch_id: None,
                    parent_record_id: Some(
                        TraceRecordId::new("record-1").expect("parent record id"),
                    ),
                },
                kind: TraceRecordKind::CompletionCheckpoint(TraceCompletionCheckpoint {
                    checkpoint_id: TraceCheckpointId::new("checkpoint-1").expect("checkpoint id"),
                    kind: TraceCheckpointKind::TurnCompleted,
                    summary: "turn completed".to_string(),
                    response: None,
                    citations: Vec::new(),
                    grounded: true,
                }),
            })
            .expect("record checkpoint");
        recorder
            .record(turn_started_record(
                &task_id,
                &turn_two,
                "record-3",
                3,
                "record-2",
                "follow-up prompt",
            ))
            .expect("record next turn");

        let wake = recorder.wake(&task_id).expect("wake");
        assert_eq!(wake.task_id, task_id);
        assert_eq!(
            wake.latest_record_id,
            Some(TraceRecordId::new("record-3").expect("record id"))
        );
        assert_eq!(wake.latest_sequence, Some(3));
        assert_eq!(wake.checkpoints.len(), 1);
        assert_eq!(
            wake.checkpoints[0].resume_request,
            TraceReplaySliceRequest::from_anchor(TraceReplaySliceAnchor::Checkpoint(
                TraceCheckpointId::new("checkpoint-1").expect("checkpoint id")
            ))
        );
    }

    #[test]
    fn replay_slice_selects_records_from_checkpoint_forward() {
        let temp = tempdir().expect("tempdir");
        let recorder = TransitTraceRecorder::open(temp.path()).expect("transit recorder");
        let task_id = TaskTraceId::new("task-slice").expect("task id");
        let turn_one = TurnTraceId::new("task-slice.turn-0001").expect("turn");
        let turn_two = TurnTraceId::new("task-slice.turn-0002").expect("turn");
        recorder
            .record(root_record("task-slice"))
            .expect("record root");
        recorder
            .record(TraceRecord {
                record_id: TraceRecordId::new("record-2").expect("record id"),
                sequence: 2,
                lineage: TraceLineage {
                    task_id: task_id.clone(),
                    turn_id: turn_one,
                    branch_id: None,
                    parent_record_id: Some(
                        TraceRecordId::new("record-1").expect("parent record id"),
                    ),
                },
                kind: TraceRecordKind::CompletionCheckpoint(TraceCompletionCheckpoint {
                    checkpoint_id: TraceCheckpointId::new("checkpoint-1").expect("checkpoint id"),
                    kind: TraceCheckpointKind::TurnCompleted,
                    summary: "turn completed".to_string(),
                    response: None,
                    citations: Vec::new(),
                    grounded: true,
                }),
            })
            .expect("record checkpoint");
        recorder
            .record(turn_started_record(
                &task_id,
                &turn_two,
                "record-3",
                3,
                "record-2",
                "second turn",
            ))
            .expect("record next turn");

        let slice = recorder
            .replay_slice(
                &task_id,
                &TraceReplaySliceRequest::from_anchor(TraceReplaySliceAnchor::Checkpoint(
                    TraceCheckpointId::new("checkpoint-1").expect("checkpoint id"),
                )),
            )
            .expect("slice");
        let record_ids = slice
            .records
            .into_iter()
            .map(|record| record.record_id)
            .collect::<Vec<_>>();
        assert_eq!(
            record_ids,
            vec![
                TraceRecordId::new("record-2").expect("record id"),
                TraceRecordId::new("record-3").expect("record id"),
            ]
        );
    }

    #[test]
    fn default_trace_recorder_prefers_embedded_transit_session_spine() {
        let workspace = tempdir().expect("workspace");
        let state_root = tempdir().expect("state root");

        let recorder =
            default_trace_recorder_for_workspace_under_root(workspace.path(), state_root.path());

        assert!(
            recorder
                .as_any()
                .downcast_ref::<TransitTraceRecorder>()
                .is_some()
        );
        assert!(matches!(
            recorder.capability(),
            TraceRecorderCapability::Persistent { .. }
        ));
    }

    #[test]
    fn default_trace_recorder_falls_back_to_in_memory_when_transit_state_is_unavailable() {
        let workspace = tempdir().expect("workspace");
        let state_root = tempdir().expect("state root");
        let blocked_root = state_root.path().join("blocked-root");
        fs::write(&blocked_root, "not a directory").expect("write blocked root");

        let recorder =
            default_trace_recorder_for_workspace_under_root(workspace.path(), &blocked_root);

        assert!(
            recorder
                .as_any()
                .downcast_ref::<InMemoryTraceRecorder>()
                .is_some()
        );
        assert!(matches!(
            recorder.capability(),
            TraceRecorderCapability::Ephemeral { backend, reason }
                if backend == "in_memory"
                    && reason.contains("embedded transit session spine")
        ));
    }
}
