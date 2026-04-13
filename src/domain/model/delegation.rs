use super::{
    ExecutionApprovalPolicy, ExecutionGovernanceSnapshot, ExecutionPermission,
    ExecutionPermissionReuseScope, ExecutionSandboxMode, TraceRecordKind, TraceReplay,
    TraceWorkerArtifact, TraceWorkerIntegration, TraceWorkerLifecycle,
};
use paddles_conversation::{ConversationThreadRef, TraceBranchId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerLifecycleOperation {
    Spawn,
    FollowUpInput,
    Wait,
    Resume,
    Close,
}

impl WorkerLifecycleOperation {
    pub fn label(self) -> &'static str {
        match self {
            Self::Spawn => "spawn",
            Self::FollowUpInput => "follow_up_input",
            Self::Wait => "wait",
            Self::Resume => "resume",
            Self::Close => "close",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerLifecycleResultStatus {
    Accepted,
    Conflict,
    Rejected,
    Stale,
    Unavailable,
}

impl WorkerLifecycleResultStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Conflict => "conflict",
            Self::Rejected => "rejected",
            Self::Stale => "stale",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerLifecycleResult {
    pub operation: WorkerLifecycleOperation,
    pub status: WorkerLifecycleResultStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_id: Option<String>,
    pub detail: String,
}

impl WorkerLifecycleResult {
    pub fn new(
        operation: WorkerLifecycleOperation,
        status: WorkerLifecycleResultStatus,
        worker_id: Option<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            operation,
            status,
            worker_id,
            detail: detail.into(),
        }
    }

    pub fn summary(&self) -> String {
        format!("{} {}", self.operation.label(), self.status.label())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerRole {
    pub id: String,
    pub label: String,
    pub summary: String,
}

impl WorkerRole {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            summary: summary.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationIntegrationOwner {
    Parent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerOwnership {
    pub summary: String,
    pub read_scopes: Vec<String>,
    pub write_scopes: Vec<String>,
    pub integration_owner: DelegationIntegrationOwner,
}

impl WorkerOwnership {
    pub fn new(
        summary: impl Into<String>,
        read_scopes: Vec<String>,
        write_scopes: Vec<String>,
        integration_owner: DelegationIntegrationOwner,
    ) -> Self {
        Self {
            summary: summary.into(),
            read_scopes: canonicalize_strings(read_scopes),
            write_scopes: canonicalize_strings(write_scopes),
            integration_owner,
        }
    }

    pub fn conflicting_write_scopes(&self, other: &Self) -> Vec<String> {
        let mut conflicts = Vec::new();
        for left in &self.write_scopes {
            for right in &other.write_scopes {
                let left = normalize_scope(left);
                let right = normalize_scope(right);
                if scopes_conflict(&left, &right) {
                    conflicts.push(if left == right || right.starts_with(&format!("{left}/")) {
                        left.clone()
                    } else {
                        right.clone()
                    });
                }
            }
        }
        canonicalize_strings(conflicts)
    }

    pub fn conflicts_with(&self, other: &Self) -> bool {
        !self.conflicting_write_scopes(other).is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerArtifactKind {
    ToolCall,
    ToolOutput,
    CompletionSummary,
}

impl WorkerArtifactKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::ToolCall => "tool_call",
            Self::ToolOutput => "tool_output",
            Self::CompletionSummary => "completion_summary",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerIntegrationStatus {
    Integrated,
    Rejected,
    Stale,
    Unavailable,
}

impl WorkerIntegrationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Integrated => "integrated",
            Self::Rejected => "rejected",
            Self::Stale => "stale",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationEvidencePolicy {
    pub summary: String,
    pub kinds: Vec<WorkerArtifactKind>,
}

impl DelegationEvidencePolicy {
    pub fn new(summary: impl Into<String>, kinds: Vec<WorkerArtifactKind>) -> Self {
        let mut kinds = kinds;
        kinds.sort();
        kinds.dedup();
        Self {
            summary: summary.into(),
            kinds,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationGovernancePolicy {
    pub requested_profile_id: String,
    pub active_profile_id: String,
    pub sandbox_mode: ExecutionSandboxMode,
    pub approval_policy: ExecutionApprovalPolicy,
    pub allowed_permissions: Vec<ExecutionPermission>,
    pub supported_reuse_scopes: Vec<ExecutionPermissionReuseScope>,
    pub downgrade_reason: Option<String>,
    pub evidence_policy: DelegationEvidencePolicy,
}

impl DelegationGovernancePolicy {
    pub fn inherit_from_parent(
        snapshot: &ExecutionGovernanceSnapshot,
        evidence_policy: DelegationEvidencePolicy,
    ) -> Self {
        Self {
            requested_profile_id: snapshot.requested_profile_id.clone(),
            active_profile_id: snapshot.active_profile_id.clone(),
            sandbox_mode: snapshot.profile.sandbox_mode,
            approval_policy: snapshot.profile.approval_policy,
            allowed_permissions: canonicalize_ord(snapshot.profile.allowed_permissions.clone()),
            supported_reuse_scopes: canonicalize_ord(
                snapshot.profile.supported_reuse_scopes.clone(),
            ),
            downgrade_reason: snapshot.profile.downgrade_reason.clone(),
            evidence_policy,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerDelegationContract {
    pub role: WorkerRole,
    pub ownership: WorkerOwnership,
    pub governance: DelegationGovernancePolicy,
}

impl WorkerDelegationContract {
    pub fn new(
        role: WorkerRole,
        ownership: WorkerOwnership,
        governance: DelegationGovernancePolicy,
    ) -> Self {
        Self {
            role,
            ownership,
            governance,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerDelegationRequest {
    pub operation: WorkerLifecycleOperation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract: Option<WorkerDelegationContract>,
}

impl WorkerDelegationRequest {
    pub fn spawn(instruction: impl Into<String>, contract: WorkerDelegationContract) -> Self {
        Self {
            operation: WorkerLifecycleOperation::Spawn,
            worker_id: None,
            instruction: Some(instruction.into()),
            contract: Some(contract),
        }
    }

    pub fn follow_up(worker_id: impl Into<String>, instruction: impl Into<String>) -> Self {
        Self {
            operation: WorkerLifecycleOperation::FollowUpInput,
            worker_id: Some(worker_id.into()),
            instruction: Some(instruction.into()),
            contract: None,
        }
    }

    pub fn wait(worker_id: impl Into<String>) -> Self {
        Self {
            operation: WorkerLifecycleOperation::Wait,
            worker_id: Some(worker_id.into()),
            instruction: None,
            contract: None,
        }
    }

    pub fn resume(worker_id: impl Into<String>) -> Self {
        Self {
            operation: WorkerLifecycleOperation::Resume,
            worker_id: Some(worker_id.into()),
            instruction: None,
            contract: None,
        }
    }

    pub fn close(worker_id: impl Into<String>) -> Self {
        Self {
            operation: WorkerLifecycleOperation::Close,
            worker_id: Some(worker_id.into()),
            instruction: None,
            contract: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerArtifactRecord {
    pub worker_id: String,
    pub kind: WorkerArtifactKind,
    pub label: String,
    pub summary: String,
    pub integration_hints: Vec<String>,
    pub parent_visible: bool,
}

impl WorkerArtifactRecord {
    pub fn tool_call(
        worker_id: impl Into<String>,
        tool_name: impl Into<String>,
        invocation: impl Into<String>,
    ) -> Self {
        Self {
            worker_id: worker_id.into(),
            kind: WorkerArtifactKind::ToolCall,
            label: tool_name.into(),
            summary: invocation.into(),
            integration_hints: Vec::new(),
            parent_visible: true,
        }
    }

    pub fn tool_output(
        worker_id: impl Into<String>,
        tool_name: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            worker_id: worker_id.into(),
            kind: WorkerArtifactKind::ToolOutput,
            label: tool_name.into(),
            summary: summary.into(),
            integration_hints: Vec::new(),
            parent_visible: true,
        }
    }

    pub fn completion_summary(
        worker_id: impl Into<String>,
        summary: impl Into<String>,
        integration_hints: Vec<String>,
    ) -> Self {
        Self {
            worker_id: worker_id.into(),
            kind: WorkerArtifactKind::CompletionSummary,
            label: WorkerArtifactKind::CompletionSummary.label().to_string(),
            summary: summary.into(),
            integration_hints: canonicalize_strings(integration_hints),
            parent_visible: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegatedWorkerStatus {
    Running,
    Waiting,
    AwaitingIntegration,
    Integrated,
    Closed,
    Conflict,
    Rejected,
    Stale,
    Unavailable,
}

impl DelegatedWorkerStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::AwaitingIntegration => "awaiting_integration",
            Self::Integrated => "integrated",
            Self::Closed => "closed",
            Self::Conflict => "conflict",
            Self::Rejected => "rejected",
            Self::Stale => "stale",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegatedWorkerSnapshot {
    pub worker_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_thread: Option<ConversationThreadRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_thread: Option<ConversationThreadRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract: Option<WorkerDelegationContract>,
    pub lifecycle: Vec<TraceWorkerLifecycle>,
    pub artifacts: Vec<TraceWorkerArtifact>,
    pub integrations: Vec<TraceWorkerIntegration>,
    pub status: DelegatedWorkerStatus,
    pub latest_detail: String,
}

impl DelegatedWorkerSnapshot {
    fn new(worker_id: String) -> Self {
        Self {
            worker_id,
            parent_thread: None,
            worker_thread: None,
            contract: None,
            lifecycle: Vec::new(),
            artifacts: Vec::new(),
            integrations: Vec::new(),
            status: DelegatedWorkerStatus::Running,
            latest_detail: String::new(),
        }
    }

    pub fn parent_can_continue(&self) -> bool {
        !matches!(self.status, DelegatedWorkerStatus::Waiting)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DelegationReplayView {
    pub workers: Vec<DelegatedWorkerSnapshot>,
}

impl DelegationReplayView {
    pub fn from_trace_replay(replay: &TraceReplay) -> Self {
        let mut workers = BTreeMap::<String, DelegatedWorkerSnapshot>::new();

        for record in &replay.records {
            match &record.kind {
                TraceRecordKind::WorkerLifecycleRecorded(lifecycle) => {
                    let Some(worker_id) = lifecycle
                        .result
                        .worker_id
                        .clone()
                        .or_else(|| lifecycle.request.worker_id.clone())
                    else {
                        continue;
                    };

                    let worker = workers
                        .entry(worker_id.clone())
                        .or_insert_with(|| DelegatedWorkerSnapshot::new(worker_id));
                    worker
                        .parent_thread
                        .get_or_insert(lifecycle.parent_thread.clone());
                    worker
                        .worker_thread
                        .get_or_insert(lifecycle.worker_thread.clone());
                    if let Some(contract) = lifecycle.request.contract.clone() {
                        worker.contract = Some(contract);
                    }
                    worker.latest_detail = lifecycle.result.detail.clone();
                    worker.status = delegated_status_from_lifecycle(&lifecycle.result);
                    worker.lifecycle.push(lifecycle.clone());
                }
                TraceRecordKind::WorkerArtifactRecorded(artifact) => {
                    let worker = workers
                        .entry(artifact.record.worker_id.clone())
                        .or_insert_with(|| {
                            DelegatedWorkerSnapshot::new(artifact.record.worker_id.clone())
                        });
                    if worker.worker_thread.is_none() {
                        worker.worker_thread =
                            Some(thread_ref_from_branch_id(record.lineage.branch_id.as_ref()));
                    }
                    worker.latest_detail = artifact.record.summary.clone();
                    if artifact.record.kind == WorkerArtifactKind::CompletionSummary
                        && !matches!(
                            worker.status,
                            DelegatedWorkerStatus::Integrated
                                | DelegatedWorkerStatus::Conflict
                                | DelegatedWorkerStatus::Rejected
                                | DelegatedWorkerStatus::Stale
                                | DelegatedWorkerStatus::Unavailable
                        )
                    {
                        worker.status = DelegatedWorkerStatus::AwaitingIntegration;
                    }
                    worker.artifacts.push(artifact.clone());
                }
                TraceRecordKind::WorkerIntegrationRecorded(integration) => {
                    let worker =
                        workers
                            .entry(integration.worker_id.clone())
                            .or_insert_with(|| {
                                DelegatedWorkerSnapshot::new(integration.worker_id.clone())
                            });
                    worker
                        .parent_thread
                        .get_or_insert(integration.parent_thread.clone());
                    worker
                        .worker_thread
                        .get_or_insert(integration.worker_thread.clone());
                    worker.latest_detail = integration.detail.clone();
                    worker.status = delegated_status_from_integration(integration.status);
                    worker.integrations.push(integration.clone());
                }
                _ => {}
            }
        }

        Self {
            workers: workers.into_values().collect(),
        }
    }
}

fn canonicalize_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn canonicalize_ord<T: Ord>(mut values: Vec<T>) -> Vec<T> {
    values.sort();
    values.dedup();
    values
}

fn delegated_status_from_lifecycle(result: &WorkerLifecycleResult) -> DelegatedWorkerStatus {
    match result.status {
        WorkerLifecycleResultStatus::Accepted => match result.operation {
            WorkerLifecycleOperation::Spawn
            | WorkerLifecycleOperation::FollowUpInput
            | WorkerLifecycleOperation::Resume => DelegatedWorkerStatus::Running,
            WorkerLifecycleOperation::Wait => DelegatedWorkerStatus::Waiting,
            WorkerLifecycleOperation::Close => DelegatedWorkerStatus::Closed,
        },
        WorkerLifecycleResultStatus::Conflict => DelegatedWorkerStatus::Conflict,
        WorkerLifecycleResultStatus::Rejected => DelegatedWorkerStatus::Rejected,
        WorkerLifecycleResultStatus::Stale => DelegatedWorkerStatus::Stale,
        WorkerLifecycleResultStatus::Unavailable => DelegatedWorkerStatus::Unavailable,
    }
}

fn delegated_status_from_integration(status: WorkerIntegrationStatus) -> DelegatedWorkerStatus {
    match status {
        WorkerIntegrationStatus::Integrated => DelegatedWorkerStatus::Integrated,
        WorkerIntegrationStatus::Rejected => DelegatedWorkerStatus::Rejected,
        WorkerIntegrationStatus::Stale => DelegatedWorkerStatus::Stale,
        WorkerIntegrationStatus::Unavailable => DelegatedWorkerStatus::Unavailable,
    }
}

fn thread_ref_from_branch_id(branch_id: Option<&TraceBranchId>) -> ConversationThreadRef {
    branch_id
        .cloned()
        .map(ConversationThreadRef::Branch)
        .unwrap_or(ConversationThreadRef::Mainline)
}

fn normalize_scope(scope: &str) -> String {
    scope.trim_matches('/').to_string()
}

fn scopes_conflict(left: &str, right: &str) -> bool {
    left == right
        || right.starts_with(&format!("{left}/"))
        || left.starts_with(&format!("{right}/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{
        ArtifactEnvelope, ArtifactKind, ConversationThreadRef, DelegatedWorkerStatus,
        ExecutionApprovalPolicy, ExecutionGovernanceProfile, ExecutionGovernanceSnapshot,
        ExecutionPermission, ExecutionPermissionReuseScope, ExecutionSandboxMode, TaskTraceId,
        TraceArtifactId, TraceBranchId, TraceLineage, TraceRecord, TraceRecordId, TraceRecordKind,
        TraceReplay, TraceWorkerArtifact, TraceWorkerIntegration, TraceWorkerLifecycle,
        TurnTraceId, WorkerIntegrationStatus,
    };
    use serde_json::json;

    #[test]
    fn worker_lifecycle_contracts_cover_spawn_follow_up_wait_resume_and_close() {
        let labels = [
            WorkerLifecycleOperation::Spawn,
            WorkerLifecycleOperation::FollowUpInput,
            WorkerLifecycleOperation::Wait,
            WorkerLifecycleOperation::Resume,
            WorkerLifecycleOperation::Close,
        ]
        .map(WorkerLifecycleOperation::label);

        assert_eq!(
            labels,
            ["spawn", "follow_up_input", "wait", "resume", "close"]
        );

        let statuses = [
            WorkerLifecycleResultStatus::Accepted,
            WorkerLifecycleResultStatus::Conflict,
            WorkerLifecycleResultStatus::Rejected,
            WorkerLifecycleResultStatus::Stale,
            WorkerLifecycleResultStatus::Unavailable,
        ]
        .map(WorkerLifecycleResultStatus::label);

        assert_eq!(
            statuses,
            ["accepted", "conflict", "rejected", "stale", "unavailable"]
        );

        let contract = WorkerDelegationContract::new(
            WorkerRole::new(
                "worker",
                "Worker",
                "Apply the bounded implementation slice.",
            ),
            WorkerOwnership::new(
                "Own src/domain/model/delegation.rs",
                vec!["src/domain/model".to_string()],
                vec!["src/domain/model/delegation.rs".to_string()],
                DelegationIntegrationOwner::Parent,
            ),
            DelegationGovernancePolicy::inherit_from_parent(
                &sample_governance_snapshot(),
                DelegationEvidencePolicy::new(
                    "Worker records stay parent-visible.",
                    vec![WorkerArtifactKind::CompletionSummary],
                ),
            ),
        );
        let spawn = WorkerDelegationRequest::spawn("Define delegation contracts", contract);
        let follow_up = WorkerDelegationRequest::follow_up("worker-1", "Refine ownership detail");
        let wait = WorkerDelegationRequest::wait("worker-1");
        let resume = WorkerDelegationRequest::resume("worker-1");
        let close = WorkerDelegationRequest::close("worker-1");

        assert_eq!(spawn.operation, WorkerLifecycleOperation::Spawn);
        assert!(spawn.contract.is_some());
        assert_eq!(follow_up.operation, WorkerLifecycleOperation::FollowUpInput);
        assert_eq!(follow_up.worker_id.as_deref(), Some("worker-1"));
        assert_eq!(wait.operation, WorkerLifecycleOperation::Wait);
        assert_eq!(resume.operation, WorkerLifecycleOperation::Resume);
        assert_eq!(close.operation, WorkerLifecycleOperation::Close);
    }

    #[test]
    fn delegation_requests_carry_role_ownership_and_parent_integration_without_surface_fields() {
        let request = WorkerDelegationRequest::spawn(
            "Investigate the parser contract",
            WorkerDelegationContract::new(
                WorkerRole::new(
                    "explorer",
                    "Explorer",
                    "Read code, inspect traces, and return bounded findings.",
                ),
                WorkerOwnership::new(
                    "Inspect parser internals without applying edits.",
                    vec!["src/domain/model".to_string()],
                    Vec::new(),
                    DelegationIntegrationOwner::Parent,
                ),
                DelegationGovernancePolicy::inherit_from_parent(
                    &sample_governance_snapshot(),
                    DelegationEvidencePolicy::new(
                        "Worker results stay visible as tool-call, output, and completion records.",
                        vec![
                            WorkerArtifactKind::ToolCall,
                            WorkerArtifactKind::ToolOutput,
                            WorkerArtifactKind::CompletionSummary,
                        ],
                    ),
                ),
            ),
        );

        let value = serde_json::to_value(&request).expect("delegation request serializes");

        assert_eq!(value["operation"], json!("spawn"));
        assert_eq!(value["contract"]["role"]["id"], json!("explorer"));
        assert_eq!(
            value["contract"]["ownership"]["integration_owner"],
            json!("parent")
        );
        assert!(value.get("provider").is_none());
        assert!(value.get("surface").is_none());
        assert!(value.get("transport").is_none());
    }

    #[test]
    fn delegation_governance_policy_inherits_parent_execution_posture_and_evidence_shape() {
        let snapshot = sample_governance_snapshot();
        let evidence_policy = DelegationEvidencePolicy::new(
            "Worker artifacts must stay visible to the parent.",
            vec![
                WorkerArtifactKind::ToolCall,
                WorkerArtifactKind::ToolOutput,
                WorkerArtifactKind::CompletionSummary,
            ],
        );

        let policy =
            DelegationGovernancePolicy::inherit_from_parent(&snapshot, evidence_policy.clone());

        assert_eq!(policy.requested_profile_id, "recursive-structured-v1");
        assert_eq!(policy.active_profile_id, "recursive-structured-v1");
        assert_eq!(policy.sandbox_mode, ExecutionSandboxMode::WorkspaceWrite);
        assert_eq!(policy.approval_policy, ExecutionApprovalPolicy::OnRequest);
        assert!(
            policy
                .allowed_permissions
                .contains(&ExecutionPermission::RunWorkspaceCommand)
        );
        assert!(
            policy
                .supported_reuse_scopes
                .contains(&ExecutionPermissionReuseScope::Hand)
        );
        assert_eq!(policy.evidence_policy, evidence_policy);
    }

    #[test]
    fn worker_artifact_records_are_parent_visible_runtime_contracts() {
        let record =
            WorkerArtifactRecord::tool_call("worker-7", "shell", "rg delegation src/domain/model");
        let completion = WorkerArtifactRecord::completion_summary(
            "worker-7",
            "Delegation contract audit complete",
            vec!["Parent integrates the findings into the main thread.".to_string()],
        );

        assert_eq!(record.kind, WorkerArtifactKind::ToolCall);
        assert!(record.parent_visible);
        assert_eq!(completion.kind, WorkerArtifactKind::CompletionSummary);
        assert!(completion.parent_visible);
        assert!(
            completion
                .integration_hints
                .contains(&"Parent integrates the findings into the main thread.".to_string())
        );
    }

    #[test]
    fn delegation_replay_preserves_wait_resume_and_integration_through_thread_lineage() {
        let task_id = TaskTraceId::new("task-delegation").expect("task");
        let turn_id = TurnTraceId::new("task-delegation.turn-0001").expect("turn");
        let worker_branch = TraceBranchId::new("worker-branch-1").expect("branch");
        let worker_thread = ConversationThreadRef::Branch(worker_branch.clone());
        let contract = WorkerDelegationContract::new(
            WorkerRole::new(
                "worker",
                "Worker",
                "Investigate the parser and return bounded findings.",
            ),
            WorkerOwnership::new(
                "Own parser traces",
                vec!["src/domain/model".to_string()],
                vec!["src/domain/model/delegation.rs".to_string()],
                DelegationIntegrationOwner::Parent,
            ),
            DelegationGovernancePolicy::inherit_from_parent(
                &sample_governance_snapshot(),
                DelegationEvidencePolicy::new(
                    "Tool calls, outputs, and summaries stay parent-visible.",
                    vec![
                        WorkerArtifactKind::ToolCall,
                        WorkerArtifactKind::ToolOutput,
                        WorkerArtifactKind::CompletionSummary,
                    ],
                ),
            ),
        );
        let replay = TraceReplay {
            task_id: task_id.clone(),
            records: vec![
                TraceRecord {
                    record_id: TraceRecordId::new("record-1").expect("record"),
                    sequence: 1,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: None,
                    },
                    kind: TraceRecordKind::WorkerLifecycleRecorded(TraceWorkerLifecycle {
                        request: WorkerDelegationRequest::spawn(
                            "Audit parser threading",
                            contract.clone(),
                        ),
                        result: WorkerLifecycleResult::new(
                            WorkerLifecycleOperation::Spawn,
                            WorkerLifecycleResultStatus::Accepted,
                            Some("worker-1".to_string()),
                            "Spawned the worker on a child thread.",
                        ),
                        parent_thread: ConversationThreadRef::Mainline,
                        worker_thread: worker_thread.clone(),
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-2").expect("record"),
                    sequence: 2,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: Some(TraceRecordId::new("record-1").expect("record")),
                    },
                    kind: TraceRecordKind::WorkerLifecycleRecorded(TraceWorkerLifecycle {
                        request: WorkerDelegationRequest::wait("worker-1"),
                        result: WorkerLifecycleResult::new(
                            WorkerLifecycleOperation::Wait,
                            WorkerLifecycleResultStatus::Accepted,
                            Some("worker-1".to_string()),
                            "Parent yielded until the worker reached a checkpoint.",
                        ),
                        parent_thread: ConversationThreadRef::Mainline,
                        worker_thread: worker_thread.clone(),
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-3").expect("record"),
                    sequence: 3,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: Some(worker_branch.clone()),
                        parent_record_id: Some(TraceRecordId::new("record-2").expect("record")),
                    },
                    kind: TraceRecordKind::WorkerArtifactRecorded(TraceWorkerArtifact {
                        record: WorkerArtifactRecord::tool_call(
                            "worker-1",
                            "shell",
                            "rg parser src/domain/model",
                        ),
                        artifact: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-1").expect("artifact"),
                            ArtifactKind::ToolInvocation,
                            "worker tool call",
                            "rg parser src/domain/model",
                            256,
                        ),
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-4").expect("record"),
                    sequence: 4,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: Some(worker_branch.clone()),
                        parent_record_id: Some(TraceRecordId::new("record-3").expect("record")),
                    },
                    kind: TraceRecordKind::WorkerArtifactRecorded(TraceWorkerArtifact {
                        record: WorkerArtifactRecord::tool_output(
                            "worker-1",
                            "shell",
                            "Found 6 parser-related trace sites.",
                        ),
                        artifact: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-2").expect("artifact"),
                            ArtifactKind::ToolOutput,
                            "worker tool output",
                            "Found 6 parser-related trace sites.",
                            256,
                        ),
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-5").expect("record"),
                    sequence: 5,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: Some(TraceRecordId::new("record-4").expect("record")),
                    },
                    kind: TraceRecordKind::WorkerLifecycleRecorded(TraceWorkerLifecycle {
                        request: WorkerDelegationRequest::resume("worker-1"),
                        result: WorkerLifecycleResult::new(
                            WorkerLifecycleOperation::Resume,
                            WorkerLifecycleResultStatus::Accepted,
                            Some("worker-1".to_string()),
                            "Parent resumed after the worker checkpoint.",
                        ),
                        parent_thread: ConversationThreadRef::Mainline,
                        worker_thread: worker_thread.clone(),
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-6").expect("record"),
                    sequence: 6,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: Some(worker_branch.clone()),
                        parent_record_id: Some(TraceRecordId::new("record-5").expect("record")),
                    },
                    kind: TraceRecordKind::WorkerArtifactRecorded(TraceWorkerArtifact {
                        record: WorkerArtifactRecord::completion_summary(
                            "worker-1",
                            "Parser audit complete",
                            vec!["Integrate the parser findings into the parent plan.".to_string()],
                        ),
                        artifact: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-3").expect("artifact"),
                            ArtifactKind::Selection,
                            "worker completion summary",
                            "Parser audit complete",
                            256,
                        ),
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-7").expect("record"),
                    sequence: 7,
                    lineage: TraceLineage {
                        task_id,
                        turn_id,
                        branch_id: None,
                        parent_record_id: Some(TraceRecordId::new("record-6").expect("record")),
                    },
                    kind: TraceRecordKind::WorkerIntegrationRecorded(TraceWorkerIntegration {
                        worker_id: "worker-1".to_string(),
                        parent_thread: ConversationThreadRef::Mainline,
                        worker_thread: worker_thread.clone(),
                        status: WorkerIntegrationStatus::Integrated,
                        detail: "Integrated the worker findings into the parent turn.".to_string(),
                        integrated_artifact_ids: vec![
                            TraceArtifactId::new("artifact-3").expect("artifact"),
                        ],
                    }),
                },
            ],
        };

        let view = DelegationReplayView::from_trace_replay(&replay);
        let worker = view.workers.first().expect("worker snapshot");

        assert_eq!(view.workers.len(), 1);
        assert_eq!(worker.worker_id, "worker-1");
        assert_eq!(worker.parent_thread, Some(ConversationThreadRef::Mainline));
        assert_eq!(worker.worker_thread, Some(worker_thread));
        assert_eq!(worker.status, DelegatedWorkerStatus::Integrated);
        assert_eq!(
            worker
                .lifecycle
                .iter()
                .map(|record| record.request.operation.label())
                .collect::<Vec<_>>(),
            vec!["spawn", "wait", "resume"]
        );
    }

    #[test]
    fn delegation_replay_keeps_worker_artifacts_parent_visible_before_integration() {
        let task_id = TaskTraceId::new("task-worker-artifacts").expect("task");
        let turn_id = TurnTraceId::new("task-worker-artifacts.turn-0001").expect("turn");
        let worker_branch = TraceBranchId::new("worker-branch-artifacts").expect("branch");
        let replay = TraceReplay {
            task_id: task_id.clone(),
            records: vec![
                TraceRecord {
                    record_id: TraceRecordId::new("record-1").expect("record"),
                    sequence: 1,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: None,
                    },
                    kind: TraceRecordKind::WorkerLifecycleRecorded(TraceWorkerLifecycle {
                        request: WorkerDelegationRequest::spawn(
                            "Inspect worker artifact replay",
                            WorkerDelegationContract::new(
                                WorkerRole::new(
                                    "explorer",
                                    "Explorer",
                                    "Inspect and summarize delegated execution.",
                                ),
                                WorkerOwnership::new(
                                    "Read-only delegated replay inspection.",
                                    vec!["src/domain/model".to_string()],
                                    Vec::new(),
                                    DelegationIntegrationOwner::Parent,
                                ),
                                DelegationGovernancePolicy::inherit_from_parent(
                                    &sample_governance_snapshot(),
                                    DelegationEvidencePolicy::new(
                                        "Worker artifacts stay parent-visible.",
                                        vec![
                                            WorkerArtifactKind::ToolCall,
                                            WorkerArtifactKind::ToolOutput,
                                            WorkerArtifactKind::CompletionSummary,
                                        ],
                                    ),
                                ),
                            ),
                        ),
                        result: WorkerLifecycleResult::new(
                            WorkerLifecycleOperation::Spawn,
                            WorkerLifecycleResultStatus::Accepted,
                            Some("worker-9".to_string()),
                            "Spawned delegated replay inspector.",
                        ),
                        parent_thread: ConversationThreadRef::Mainline,
                        worker_thread: ConversationThreadRef::Branch(worker_branch.clone()),
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-2").expect("record"),
                    sequence: 2,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: Some(worker_branch.clone()),
                        parent_record_id: Some(TraceRecordId::new("record-1").expect("record")),
                    },
                    kind: TraceRecordKind::WorkerArtifactRecorded(TraceWorkerArtifact {
                        record: WorkerArtifactRecord::tool_call(
                            "worker-9",
                            "shell",
                            "cargo test delegation --quiet",
                        ),
                        artifact: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-call").expect("artifact"),
                            ArtifactKind::ToolInvocation,
                            "worker tool call",
                            "cargo test delegation --quiet",
                            256,
                        ),
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-3").expect("record"),
                    sequence: 3,
                    lineage: TraceLineage {
                        task_id,
                        turn_id,
                        branch_id: Some(worker_branch),
                        parent_record_id: Some(TraceRecordId::new("record-2").expect("record")),
                    },
                    kind: TraceRecordKind::WorkerArtifactRecorded(TraceWorkerArtifact {
                        record: WorkerArtifactRecord::completion_summary(
                            "worker-9",
                            "Delegated replay inspection complete",
                            vec![
                                "Integrate the visibility findings into the parent turn."
                                    .to_string(),
                            ],
                        ),
                        artifact: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-summary").expect("artifact"),
                            ArtifactKind::Selection,
                            "worker completion summary",
                            "Delegated replay inspection complete",
                            256,
                        ),
                    }),
                },
            ],
        };

        let view = DelegationReplayView::from_trace_replay(&replay);
        let worker = view.workers.first().expect("worker snapshot");

        assert_eq!(worker.status, DelegatedWorkerStatus::AwaitingIntegration);
        assert_eq!(worker.artifacts.len(), 2);
        assert!(
            worker
                .artifacts
                .iter()
                .all(|artifact| artifact.record.parent_visible)
        );
        assert_eq!(worker.latest_detail, "Delegated replay inspection complete");
    }

    #[test]
    fn ownership_conflicts_and_conflicting_lifecycle_results_stay_explicit() {
        let baseline = WorkerOwnership::new(
            "Own delegation core",
            vec!["src/domain/model".to_string()],
            vec!["src/domain/model/delegation.rs".to_string()],
            DelegationIntegrationOwner::Parent,
        );
        let conflicting = WorkerOwnership::new(
            "Touch a nested delegation module",
            vec!["src/domain/model".to_string()],
            vec!["src/domain/model/delegation.rs/tests".to_string()],
            DelegationIntegrationOwner::Parent,
        );

        assert_eq!(
            baseline.conflicting_write_scopes(&conflicting),
            vec!["src/domain/model/delegation.rs".to_string()]
        );

        let replay = TraceReplay {
            task_id: TaskTraceId::new("task-conflict").expect("task"),
            records: vec![TraceRecord {
                record_id: TraceRecordId::new("record-1").expect("record"),
                sequence: 1,
                lineage: TraceLineage {
                    task_id: TaskTraceId::new("task-conflict").expect("task"),
                    turn_id: TurnTraceId::new("task-conflict.turn-0001").expect("turn"),
                    branch_id: None,
                    parent_record_id: None,
                },
                kind: TraceRecordKind::WorkerLifecycleRecorded(TraceWorkerLifecycle {
                    request: WorkerDelegationRequest::spawn(
                        "Take over the same write scope",
                        WorkerDelegationContract::new(
                            WorkerRole::new("worker", "Worker", "Attempt a conflicting write."),
                            conflicting.clone(),
                            DelegationGovernancePolicy::inherit_from_parent(
                                &sample_governance_snapshot(),
                                DelegationEvidencePolicy::new(
                                    "Conflicting requests still stay visible.",
                                    vec![WorkerArtifactKind::CompletionSummary],
                                ),
                            ),
                        ),
                    ),
                    result: WorkerLifecycleResult::new(
                        WorkerLifecycleOperation::Spawn,
                        WorkerLifecycleResultStatus::Conflict,
                        Some("worker-2".to_string()),
                        "Ownership conflict: src/domain/model/delegation.rs",
                    ),
                    parent_thread: ConversationThreadRef::Mainline,
                    worker_thread: ConversationThreadRef::Mainline,
                }),
            }],
        };

        let view = DelegationReplayView::from_trace_replay(&replay);
        let worker = view.workers.first().expect("worker snapshot");

        assert_eq!(worker.status, DelegatedWorkerStatus::Conflict);
        assert_eq!(
            worker
                .contract
                .as_ref()
                .expect("contract")
                .ownership
                .write_scopes,
            conflicting.write_scopes
        );
        assert_eq!(
            worker.latest_detail,
            "Ownership conflict: src/domain/model/delegation.rs"
        );
    }

    fn sample_governance_snapshot() -> ExecutionGovernanceSnapshot {
        ExecutionGovernanceSnapshot::new(
            "recursive-structured-v1",
            "recursive-structured-v1",
            ExecutionGovernanceProfile::new(
                ExecutionSandboxMode::WorkspaceWrite,
                ExecutionApprovalPolicy::OnRequest,
                vec![
                    ExecutionPermissionReuseScope::Turn,
                    ExecutionPermissionReuseScope::CommandPrefix,
                    ExecutionPermissionReuseScope::Hand,
                ],
                None,
            ),
        )
    }
}
