use crate::domain::model::{
    ConversationThreadRef, ExecutionGovernanceDecision, TaskTraceId, TraceArtifactId, TurnTraceId,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "record", rename_all = "snake_case")]
pub enum SessionStoreRecordKind {
    Turn(SessionTurnRecord),
    PlannerDecision(SessionPlannerDecisionRecord),
    Evidence(SessionEvidenceRecord),
    Governance(Box<SessionGovernanceRecord>),
}

impl SessionStoreRecordKind {
    fn task_id(&self) -> &TaskTraceId {
        match self {
            Self::Turn(record) => &record.task_id,
            Self::PlannerDecision(record) => &record.task_id,
            Self::Evidence(record) => &record.task_id,
            Self::Governance(record) => &record.task_id,
        }
    }

    fn turn_id(&self) -> &TurnTraceId {
        match self {
            Self::Turn(record) => &record.turn_id,
            Self::PlannerDecision(record) => &record.turn_id,
            Self::Evidence(record) => &record.turn_id,
            Self::Governance(record) => &record.turn_id,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{
        ConversationThreadRef, ExecutionGovernanceDecision, ExecutionGovernanceOutcome,
        ExecutionHandKind, ExecutionPermission, ExecutionPermissionRequest,
        ExecutionPermissionRequirement, TaskTraceId, TraceArtifactId, TurnTraceId,
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
