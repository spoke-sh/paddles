use crate::domain::model::{
    ConversationThreadRef, ExecutionGovernanceDecision, TaskTraceId, TraceArtifactId,
    TraceRecordId, TurnTraceId,
};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

pub const SESSION_STORE_SCHEMA: &str = "paddles.session_store.v1";
pub const SESSION_STORE_SCHEMA_VERSION: u32 = 1;

pub trait SessionStorePort: Send + Sync {
    fn persist_record(&self, record: VersionedSessionStoreRecord) -> Result<()>;

    fn load_session(&self, task_id: &TaskTraceId) -> Result<SessionStoreSnapshot>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTurnRecord {
    pub task_id: TaskTraceId,
    pub turn_id: TurnTraceId,
    pub thread: ConversationThreadRef,
    pub prompt_summary: String,
    pub response_summary: Option<String>,
}

impl SessionTurnRecord {
    pub fn new(
        task_id: TaskTraceId,
        turn_id: TurnTraceId,
        thread: ConversationThreadRef,
        prompt_summary: impl Into<String>,
        response_summary: Option<String>,
    ) -> Self {
        Self {
            task_id,
            turn_id,
            thread,
            prompt_summary: prompt_summary.into(),
            response_summary,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPlannerDecisionRecord {
    pub task_id: TaskTraceId,
    pub turn_id: TurnTraceId,
    pub action: String,
    pub rationale: Option<String>,
}

impl SessionPlannerDecisionRecord {
    pub fn new(
        task_id: TaskTraceId,
        turn_id: TurnTraceId,
        action: impl Into<String>,
        rationale: Option<String>,
    ) -> Self {
        Self {
            task_id,
            turn_id,
            action: action.into(),
            rationale,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionEvidenceRecord {
    pub task_id: TaskTraceId,
    pub turn_id: TurnTraceId,
    pub artifact_id: TraceArtifactId,
    pub source: String,
    pub summary: String,
}

impl SessionEvidenceRecord {
    pub fn new(
        task_id: TaskTraceId,
        turn_id: TurnTraceId,
        artifact_id: TraceArtifactId,
        source: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            task_id,
            turn_id,
            artifact_id,
            source: source.into(),
            summary: summary.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionGovernanceRecord {
    pub task_id: TaskTraceId,
    pub turn_id: TurnTraceId,
    pub decision: ExecutionGovernanceDecision,
}

impl SessionGovernanceRecord {
    pub fn new(
        task_id: TaskTraceId,
        turn_id: TurnTraceId,
        decision: ExecutionGovernanceDecision,
    ) -> Self {
        Self {
            task_id,
            turn_id,
            decision,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionSnapshotStatus {
    Complete,
    Missing,
    Incomplete,
}

impl SessionSnapshotStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Missing => "missing",
            Self::Incomplete => "incomplete",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRollbackAnchor {
    pub anchor_record_id: TraceRecordId,
    pub summary: String,
}

impl SessionRollbackAnchor {
    pub fn new(anchor_record_id: TraceRecordId, summary: impl Into<String>) -> Self {
        Self {
            anchor_record_id,
            summary: summary.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSnapshotRecord {
    pub task_id: TaskTraceId,
    pub turn_id: TurnTraceId,
    pub action_id: String,
    pub affected_paths: Vec<String>,
    pub status: SessionSnapshotStatus,
    pub snapshot_artifact_id: Option<TraceArtifactId>,
    pub rollback_anchor: Option<SessionRollbackAnchor>,
    pub detail: String,
}

impl SessionSnapshotRecord {
    pub fn complete(
        task_id: TaskTraceId,
        turn_id: TurnTraceId,
        action_id: impl Into<String>,
        affected_paths: Vec<String>,
        snapshot_artifact_id: TraceArtifactId,
        rollback_anchor: SessionRollbackAnchor,
    ) -> Self {
        Self {
            task_id,
            turn_id,
            action_id: action_id.into(),
            affected_paths: Self::normalize_affected_paths(affected_paths),
            status: SessionSnapshotStatus::Complete,
            snapshot_artifact_id: Some(snapshot_artifact_id),
            rollback_anchor: Some(rollback_anchor),
            detail: "workspace snapshot and rollback anchor recorded".to_string(),
        }
    }

    pub fn missing(
        task_id: TaskTraceId,
        turn_id: TurnTraceId,
        action_id: impl Into<String>,
        affected_paths: Vec<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            task_id,
            turn_id,
            action_id: action_id.into(),
            affected_paths: Self::normalize_affected_paths(affected_paths),
            status: SessionSnapshotStatus::Missing,
            snapshot_artifact_id: None,
            rollback_anchor: None,
            detail: detail.into(),
        }
    }

    pub fn incomplete(
        task_id: TaskTraceId,
        turn_id: TurnTraceId,
        action_id: impl Into<String>,
        affected_paths: Vec<String>,
        snapshot_artifact_id: Option<TraceArtifactId>,
        rollback_anchor: Option<SessionRollbackAnchor>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            task_id,
            turn_id,
            action_id: action_id.into(),
            affected_paths: Self::normalize_affected_paths(affected_paths),
            status: SessionSnapshotStatus::Incomplete,
            snapshot_artifact_id,
            rollback_anchor,
            detail: detail.into(),
        }
    }

    fn normalize_affected_paths(mut affected_paths: Vec<String>) -> Vec<String> {
        affected_paths.sort();
        affected_paths.dedup();
        affected_paths
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionSnapshotReplayValidation {
    pub action_id: String,
    pub affected_paths: Vec<String>,
    pub status: SessionSnapshotStatus,
    pub detail: String,
    pub rollback_available: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionReplayRecord {
    pub task_id: TaskTraceId,
    pub turn_id: TurnTraceId,
    pub sequence: u64,
    pub thread: ConversationThreadRef,
    pub source_record_id: TraceRecordId,
    pub forked_from_record_id: Option<TraceRecordId>,
    pub model_visible_summary: String,
    pub evidence_artifact_ids: Vec<TraceArtifactId>,
}

impl SessionReplayRecord {
    pub fn mainline(
        task_id: TaskTraceId,
        turn_id: TurnTraceId,
        sequence: u64,
        source_record_id: TraceRecordId,
        model_visible_summary: impl Into<String>,
    ) -> Self {
        Self {
            task_id,
            turn_id,
            sequence,
            thread: ConversationThreadRef::Mainline,
            source_record_id,
            forked_from_record_id: None,
            model_visible_summary: model_visible_summary.into(),
            evidence_artifact_ids: Vec::new(),
        }
    }

    pub fn forked(
        task_id: TaskTraceId,
        turn_id: TurnTraceId,
        sequence: u64,
        thread: ConversationThreadRef,
        source_record_id: TraceRecordId,
        forked_from_record_id: TraceRecordId,
        model_visible_summary: impl Into<String>,
    ) -> Self {
        Self {
            task_id,
            turn_id,
            sequence,
            thread,
            source_record_id,
            forked_from_record_id: Some(forked_from_record_id),
            model_visible_summary: model_visible_summary.into(),
            evidence_artifact_ids: Vec::new(),
        }
    }

    pub fn with_evidence(mut self, evidence_artifact_ids: Vec<TraceArtifactId>) -> Self {
        self.evidence_artifact_ids = evidence_artifact_ids;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionModelVisibleContextEntry {
    pub turn_id: TurnTraceId,
    pub sequence: u64,
    pub thread: ConversationThreadRef,
    pub source_record_id: TraceRecordId,
    pub forked_from_record_id: Option<TraceRecordId>,
    pub model_visible_summary: String,
    pub evidence_artifact_ids: Vec<TraceArtifactId>,
}

impl From<&SessionReplayRecord> for SessionModelVisibleContextEntry {
    fn from(record: &SessionReplayRecord) -> Self {
        Self {
            turn_id: record.turn_id.clone(),
            sequence: record.sequence,
            thread: record.thread.clone(),
            source_record_id: record.source_record_id.clone(),
            forked_from_record_id: record.forked_from_record_id.clone(),
            model_visible_summary: record.model_visible_summary.clone(),
            evidence_artifact_ids: record.evidence_artifact_ids.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCompactionRecord {
    pub task_id: TaskTraceId,
    pub turn_id: TurnTraceId,
    pub summary_artifact_id: TraceArtifactId,
    pub summary: String,
    pub source_turn_ids: Vec<TurnTraceId>,
    pub source_evidence_artifact_ids: Vec<TraceArtifactId>,
    pub source_record_ids: Vec<TraceRecordId>,
}

impl SessionCompactionRecord {
    pub fn new(
        task_id: TaskTraceId,
        turn_id: TurnTraceId,
        summary_artifact_id: TraceArtifactId,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            task_id,
            turn_id,
            summary_artifact_id,
            summary: summary.into(),
            source_turn_ids: Vec::new(),
            source_evidence_artifact_ids: Vec::new(),
            source_record_ids: Vec::new(),
        }
    }

    pub fn with_source_turns(mut self, source_turn_ids: Vec<TurnTraceId>) -> Self {
        self.source_turn_ids = source_turn_ids;
        self
    }

    pub fn with_source_evidence(
        mut self,
        source_evidence_artifact_ids: Vec<TraceArtifactId>,
    ) -> Self {
        self.source_evidence_artifact_ids = source_evidence_artifact_ids;
        self
    }

    pub fn with_source_records(mut self, source_record_ids: Vec<TraceRecordId>) -> Self {
        self.source_record_ids = source_record_ids;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionCompactionLineage {
    pub summary_artifact_id: TraceArtifactId,
    pub summary: String,
    pub source_turn_ids: Vec<TurnTraceId>,
    pub source_evidence_artifact_ids: Vec<TraceArtifactId>,
    pub source_record_ids: Vec<TraceRecordId>,
}

impl From<&SessionCompactionRecord> for SessionCompactionLineage {
    fn from(record: &SessionCompactionRecord) -> Self {
        Self {
            summary_artifact_id: record.summary_artifact_id.clone(),
            summary: record.summary.clone(),
            source_turn_ids: record.source_turn_ids.clone(),
            source_evidence_artifact_ids: record.source_evidence_artifact_ids.clone(),
            source_record_ids: record.source_record_ids.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "record", rename_all = "snake_case")]
pub enum SessionStoreRecordKind {
    Turn(SessionTurnRecord),
    PlannerDecision(SessionPlannerDecisionRecord),
    Evidence(SessionEvidenceRecord),
    Governance(Box<SessionGovernanceRecord>),
    Snapshot(Box<SessionSnapshotRecord>),
    Replay(Box<SessionReplayRecord>),
    Compaction(Box<SessionCompactionRecord>),
}

impl SessionStoreRecordKind {
    fn task_id(&self) -> &TaskTraceId {
        match self {
            Self::Turn(record) => &record.task_id,
            Self::PlannerDecision(record) => &record.task_id,
            Self::Evidence(record) => &record.task_id,
            Self::Governance(record) => &record.task_id,
            Self::Snapshot(record) => &record.task_id,
            Self::Replay(record) => &record.task_id,
            Self::Compaction(record) => &record.task_id,
        }
    }

    fn turn_id(&self) -> &TurnTraceId {
        match self {
            Self::Turn(record) => &record.turn_id,
            Self::PlannerDecision(record) => &record.turn_id,
            Self::Evidence(record) => &record.turn_id,
            Self::Governance(record) => &record.turn_id,
            Self::Snapshot(record) => &record.turn_id,
            Self::Replay(record) => &record.turn_id,
            Self::Compaction(record) => &record.turn_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionStoreRecord {
    pub task_id: TaskTraceId,
    pub turn_id: TurnTraceId,
    pub kind: SessionStoreRecordKind,
}

impl SessionStoreRecord {
    pub fn new(kind: SessionStoreRecordKind) -> Self {
        Self {
            task_id: kind.task_id().clone(),
            turn_id: kind.turn_id().clone(),
            kind,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionedSessionStoreRecord {
    pub schema: String,
    pub schema_version: u32,
    pub record: SessionStoreRecord,
}

impl VersionedSessionStoreRecord {
    pub fn new(kind: SessionStoreRecordKind) -> Self {
        Self {
            schema: SESSION_STORE_SCHEMA.to_string(),
            schema_version: SESSION_STORE_SCHEMA_VERSION,
            record: SessionStoreRecord::new(kind),
        }
    }

    pub fn turn(record: SessionTurnRecord) -> Self {
        Self::new(SessionStoreRecordKind::Turn(record))
    }

    pub fn planner(record: SessionPlannerDecisionRecord) -> Self {
        Self::new(SessionStoreRecordKind::PlannerDecision(record))
    }

    pub fn evidence(record: SessionEvidenceRecord) -> Self {
        Self::new(SessionStoreRecordKind::Evidence(record))
    }

    pub fn governance(record: SessionGovernanceRecord) -> Self {
        Self::new(SessionStoreRecordKind::Governance(Box::new(record)))
    }

    pub fn snapshot(record: SessionSnapshotRecord) -> Self {
        Self::new(SessionStoreRecordKind::Snapshot(Box::new(record)))
    }

    pub fn replay(record: SessionReplayRecord) -> Self {
        Self::new(SessionStoreRecordKind::Replay(Box::new(record)))
    }

    pub fn compaction(record: SessionCompactionRecord) -> Self {
        Self::new(SessionStoreRecordKind::Compaction(Box::new(record)))
    }

    pub fn task_id(&self) -> &TaskTraceId {
        &self.record.task_id
    }

    pub fn validate_schema(&self) -> Result<()> {
        ensure!(
            self.schema == SESSION_STORE_SCHEMA
                && self.schema_version == SESSION_STORE_SCHEMA_VERSION,
            "unsupported session store schema {}@{}",
            self.schema,
            self.schema_version
        );
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionStoreSnapshot {
    pub task_id: TaskTraceId,
    pub records: Vec<VersionedSessionStoreRecord>,
}

impl SessionStoreSnapshot {
    pub fn new(task_id: TaskTraceId, records: Vec<VersionedSessionStoreRecord>) -> Self {
        Self { task_id, records }
    }

    pub fn turns(&self) -> Vec<&SessionTurnRecord> {
        self.records
            .iter()
            .filter_map(|record| match &record.record.kind {
                SessionStoreRecordKind::Turn(turn) => Some(turn),
                _ => None,
            })
            .collect()
    }

    pub fn planner_decisions(&self) -> Vec<&SessionPlannerDecisionRecord> {
        self.records
            .iter()
            .filter_map(|record| match &record.record.kind {
                SessionStoreRecordKind::PlannerDecision(decision) => Some(decision),
                _ => None,
            })
            .collect()
    }

    pub fn evidence(&self) -> Vec<&SessionEvidenceRecord> {
        self.records
            .iter()
            .filter_map(|record| match &record.record.kind {
                SessionStoreRecordKind::Evidence(evidence) => Some(evidence),
                _ => None,
            })
            .collect()
    }

    pub fn governance(&self) -> Vec<&SessionGovernanceRecord> {
        self.records
            .iter()
            .filter_map(|record| match &record.record.kind {
                SessionStoreRecordKind::Governance(governance) => Some(governance.as_ref()),
                _ => None,
            })
            .collect()
    }

    pub fn snapshots(&self) -> Vec<&SessionSnapshotRecord> {
        self.records
            .iter()
            .filter_map(|record| match &record.record.kind {
                SessionStoreRecordKind::Snapshot(snapshot) => Some(snapshot.as_ref()),
                _ => None,
            })
            .collect()
    }

    pub fn snapshot_replay_validation(&self) -> Vec<SessionSnapshotReplayValidation> {
        self.snapshots()
            .into_iter()
            .map(|snapshot| SessionSnapshotReplayValidation {
                action_id: snapshot.action_id.clone(),
                affected_paths: snapshot.affected_paths.clone(),
                status: snapshot.status,
                detail: snapshot.detail.clone(),
                rollback_available: snapshot.rollback_anchor.is_some(),
            })
            .collect()
    }

    pub fn replay_records(&self) -> Vec<&SessionReplayRecord> {
        self.records
            .iter()
            .filter_map(|record| match &record.record.kind {
                SessionStoreRecordKind::Replay(replay) => Some(replay.as_ref()),
                _ => None,
            })
            .collect()
    }

    pub fn model_visible_context(&self) -> Vec<SessionModelVisibleContextEntry> {
        let mut context = self
            .replay_records()
            .into_iter()
            .map(SessionModelVisibleContextEntry::from)
            .collect::<Vec<_>>();
        context.sort_by_key(|entry| entry.sequence);
        context
    }

    pub fn compactions(&self) -> Vec<&SessionCompactionRecord> {
        self.records
            .iter()
            .filter_map(|record| match &record.record.kind {
                SessionStoreRecordKind::Compaction(compaction) => Some(compaction.as_ref()),
                _ => None,
            })
            .collect()
    }

    pub fn compaction_lineage(&self) -> Vec<SessionCompactionLineage> {
        self.compactions()
            .into_iter()
            .map(SessionCompactionLineage::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{
        ConversationThreadRef, ExecutionGovernanceDecision, ExecutionGovernanceOutcome,
        ExecutionHandKind, ExecutionPermission, ExecutionPermissionRequest,
        ExecutionPermissionRequirement, TaskTraceId, TraceArtifactId, TraceBranchId, TraceRecordId,
        TurnTraceId,
    };
    use anyhow::Result;
    use std::sync::Mutex;

    #[derive(Default)]
    struct InMemorySessionStore {
        records: Mutex<Vec<VersionedSessionStoreRecord>>,
    }

    impl SessionStorePort for InMemorySessionStore {
        fn persist_record(&self, record: VersionedSessionStoreRecord) -> Result<()> {
            self.records.lock().expect("records").push(record);
            Ok(())
        }

        fn load_session(&self, task_id: &TaskTraceId) -> Result<SessionStoreSnapshot> {
            let records = self
                .records
                .lock()
                .expect("records")
                .iter()
                .filter(|record| record.task_id() == task_id)
                .cloned()
                .collect();
            Ok(SessionStoreSnapshot::new(task_id.clone(), records))
        }
    }

    #[test]
    fn session_store_contract_persists_and_reloads_turn_evidence_planner_and_governance_records() {
        let store = InMemorySessionStore::default();
        let task_id = TaskTraceId::new("session-task").expect("task");
        let turn_id = TurnTraceId::new("session-task.turn-0001").expect("turn");

        for record in [
            VersionedSessionStoreRecord::turn(SessionTurnRecord::new(
                task_id.clone(),
                turn_id.clone(),
                ConversationThreadRef::Mainline,
                "Implement session store contracts",
                Some("Contracts persisted locally.".to_string()),
            )),
            VersionedSessionStoreRecord::planner(SessionPlannerDecisionRecord::new(
                task_id.clone(),
                turn_id.clone(),
                "persist_records",
                Some("Planner selected local session persistence.".to_string()),
            )),
            VersionedSessionStoreRecord::evidence(SessionEvidenceRecord::new(
                task_id.clone(),
                turn_id.clone(),
                TraceArtifactId::new("evidence-1").expect("artifact"),
                "worker:session-store",
                "Evidence survives reload.",
            )),
            VersionedSessionStoreRecord::governance(SessionGovernanceRecord::new(
                task_id.clone(),
                turn_id.clone(),
                sample_governance_decision(),
            )),
        ] {
            store
                .persist_record(record)
                .expect("persist session record");
        }

        let loaded = store.load_session(&task_id).expect("load session");

        assert_eq!(loaded.task_id, task_id);
        assert_eq!(loaded.records.len(), 4);
        assert_eq!(loaded.turns().len(), 1);
        assert_eq!(loaded.planner_decisions().len(), 1);
        assert_eq!(loaded.evidence().len(), 1);
        assert_eq!(loaded.governance().len(), 1);
        assert!(
            loaded
                .records
                .iter()
                .all(|record| record.schema == SESSION_STORE_SCHEMA)
        );
    }

    #[test]
    fn session_store_versioning_attaches_schema_metadata_and_rejects_unsupported_versions() {
        let task_id = TaskTraceId::new("session-task").expect("task");
        let turn_id = TurnTraceId::new("session-task.turn-0001").expect("turn");
        let record = VersionedSessionStoreRecord::turn(SessionTurnRecord::new(
            task_id,
            turn_id,
            ConversationThreadRef::Mainline,
            "Check schema version",
            None,
        ));

        assert_eq!(record.schema, SESSION_STORE_SCHEMA);
        assert_eq!(record.schema_version, SESSION_STORE_SCHEMA_VERSION);
        assert!(record.validate_schema().is_ok());

        let mut unsupported = record.clone();
        unsupported.schema_version = SESSION_STORE_SCHEMA_VERSION + 1;
        let error = unsupported
            .validate_schema()
            .expect_err("future schema should be rejected");

        assert!(
            error
                .to_string()
                .contains("unsupported session store schema")
        );
    }

    #[test]
    fn session_snapshots_record_workspace_action_metadata_and_rollback_anchors() {
        let store = InMemorySessionStore::default();
        let task_id = TaskTraceId::new("session-task").expect("task");
        let turn_id = TurnTraceId::new("session-task.turn-0001").expect("turn");
        let rollback = SessionRollbackAnchor::new(
            TraceRecordId::new("record-before-edit").expect("record"),
            "rollback to pre-edit trace checkpoint",
        );
        let snapshot = SessionSnapshotRecord::complete(
            task_id.clone(),
            turn_id.clone(),
            "workspace-edit-1",
            vec!["src/domain/ports/session_store.rs".to_string()],
            TraceArtifactId::new("snapshot-1").expect("snapshot"),
            rollback.clone(),
        );

        store
            .persist_record(VersionedSessionStoreRecord::snapshot(snapshot))
            .expect("persist snapshot");

        let loaded = store.load_session(&task_id).expect("load session");
        let snapshots = loaded.snapshots();
        let recorded = snapshots.first().expect("snapshot record");

        assert_eq!(recorded.status, SessionSnapshotStatus::Complete);
        assert_eq!(recorded.rollback_anchor.as_ref(), Some(&rollback));
        assert_eq!(
            recorded.affected_paths,
            vec!["src/domain/ports/session_store.rs".to_string()]
        );
        assert_eq!(
            loaded.snapshot_replay_validation()[0].status,
            SessionSnapshotStatus::Complete
        );
    }

    #[test]
    fn session_snapshot_replay_validation_represents_missing_and_incomplete_snapshots_explicitly() {
        let task_id = TaskTraceId::new("session-task").expect("task");
        let turn_id = TurnTraceId::new("session-task.turn-0001").expect("turn");
        let missing = SessionSnapshotRecord::missing(
            task_id.clone(),
            turn_id.clone(),
            "workspace-edit-missing",
            vec!["src/application/mod.rs".to_string()],
            "snapshot artifact was not recorded",
        );
        let incomplete = SessionSnapshotRecord::incomplete(
            task_id.clone(),
            turn_id,
            "workspace-edit-incomplete",
            vec!["src/application/worker_runtime.rs".to_string()],
            Some(TraceArtifactId::new("snapshot-incomplete").expect("snapshot")),
            None,
            "rollback anchor was not recorded",
        );
        let snapshot = SessionStoreSnapshot::new(
            task_id,
            vec![
                VersionedSessionStoreRecord::snapshot(missing),
                VersionedSessionStoreRecord::snapshot(incomplete),
            ],
        );

        let validation = snapshot.snapshot_replay_validation();

        assert_eq!(validation.len(), 2);
        assert!(validation.iter().any(|entry| {
            entry.status == SessionSnapshotStatus::Missing
                && !entry.rollback_available
                && entry.detail.contains("snapshot artifact was not recorded")
        }));
        assert!(validation.iter().any(|entry| {
            entry.status == SessionSnapshotStatus::Incomplete
                && !entry.rollback_available
                && entry.detail.contains("rollback anchor was not recorded")
        }));
    }

    #[test]
    fn session_replay_reconstructs_model_visible_context_from_replay_metadata() {
        let store = InMemorySessionStore::default();
        let task_id = TaskTraceId::new("session-task").expect("task");
        let first_turn = TurnTraceId::new("session-task.turn-0001").expect("turn");
        let forked_turn = TurnTraceId::new("session-task.turn-0002").expect("turn");
        let root_record = TraceRecordId::new("record-root").expect("record");
        let fork_record = TraceRecordId::new("record-fork").expect("record");

        let forked = SessionReplayRecord::forked(
            task_id.clone(),
            forked_turn.clone(),
            2,
            ConversationThreadRef::Branch(TraceBranchId::new("thread-analysis").expect("branch")),
            fork_record.clone(),
            root_record.clone(),
            "Forked analysis keeps failed test evidence visible.",
        )
        .with_evidence(vec![
            TraceArtifactId::new("evidence-test").expect("artifact"),
        ]);
        let mainline = SessionReplayRecord::mainline(
            task_id.clone(),
            first_turn.clone(),
            1,
            root_record.clone(),
            "User asked for replayable local session storage.",
        );

        store
            .persist_record(VersionedSessionStoreRecord::replay(forked))
            .expect("persist forked replay");
        store
            .persist_record(VersionedSessionStoreRecord::replay(mainline))
            .expect("persist mainline replay");

        let context = store
            .load_session(&task_id)
            .expect("load session")
            .model_visible_context();

        assert_eq!(context.len(), 2);
        assert_eq!(context[0].turn_id, first_turn);
        assert_eq!(context[0].source_record_id, root_record);
        assert_eq!(
            context[1].forked_from_record_id.as_ref(),
            Some(&TraceRecordId::new("record-root").expect("record"))
        );
        assert_eq!(
            context[1].evidence_artifact_ids,
            vec![TraceArtifactId::new("evidence-test").expect("artifact")]
        );
    }

    #[test]
    fn session_compaction_lineage_links_summaries_to_source_turns_and_evidence() {
        let task_id = TaskTraceId::new("session-task").expect("task");
        let compacted_turn = TurnTraceId::new("session-task.turn-0003").expect("turn");
        let source_turn = TurnTraceId::new("session-task.turn-0001").expect("turn");
        let source_evidence = TraceArtifactId::new("evidence-1").expect("artifact");
        let source_record = TraceRecordId::new("record-source").expect("record");
        let summary_artifact = TraceArtifactId::new("summary-1").expect("artifact");
        let compaction = SessionCompactionRecord::new(
            task_id.clone(),
            compacted_turn,
            summary_artifact.clone(),
            "Summarized earlier investigation and verification evidence.",
        )
        .with_source_turns(vec![source_turn.clone()])
        .with_source_evidence(vec![source_evidence.clone()])
        .with_source_records(vec![source_record.clone()]);
        let snapshot = SessionStoreSnapshot::new(
            task_id,
            vec![VersionedSessionStoreRecord::compaction(compaction)],
        );

        let lineage = snapshot.compaction_lineage();

        assert_eq!(lineage.len(), 1);
        assert_eq!(lineage[0].summary_artifact_id, summary_artifact);
        assert_eq!(lineage[0].source_turn_ids, vec![source_turn]);
        assert_eq!(
            lineage[0].source_evidence_artifact_ids,
            vec![source_evidence]
        );
        assert_eq!(lineage[0].source_record_ids, vec![source_record]);
    }

    fn sample_governance_decision() -> ExecutionGovernanceDecision {
        let requirement = ExecutionPermissionRequirement::new(
            "Run local verification",
            vec![ExecutionPermission::RunWorkspaceCommand],
        );
        ExecutionGovernanceDecision::new(
            Some("call-1".to_string()),
            Some("shell".to_string()),
            ExecutionPermissionRequest::new(ExecutionHandKind::TerminalRunner, requirement.clone()),
            ExecutionGovernanceOutcome::allowed(
                "local verification allowed",
                requirement,
                vec![ExecutionPermission::RunWorkspaceCommand],
            ),
        )
    }
}
