use super::{
    ExecutionApprovalPolicy, ExecutionGovernanceSnapshot, ExecutionPermission,
    ExecutionPermissionReuseScope, ExecutionSandboxMode,
};
use serde::{Deserialize, Serialize};

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
    Rejected,
    Stale,
    Unavailable,
}

impl WorkerLifecycleResultStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{
        ExecutionApprovalPolicy, ExecutionGovernanceProfile, ExecutionGovernanceSnapshot,
        ExecutionPermission, ExecutionPermissionReuseScope, ExecutionSandboxMode,
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
            WorkerLifecycleResultStatus::Rejected,
            WorkerLifecycleResultStatus::Stale,
            WorkerLifecycleResultStatus::Unavailable,
        ]
        .map(WorkerLifecycleResultStatus::label);

        assert_eq!(statuses, ["accepted", "rejected", "stale", "unavailable"]);

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
