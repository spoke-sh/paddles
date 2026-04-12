use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionSandboxMode {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

impl ExecutionSandboxMode {
    pub const ALL: [Self; 3] = [Self::ReadOnly, Self::WorkspaceWrite, Self::DangerFullAccess];

    pub fn label(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::WorkspaceWrite => "workspace_write",
            Self::DangerFullAccess => "danger_full_access",
        }
    }

    pub fn default_permissions(self) -> Vec<ExecutionPermission> {
        match self {
            Self::ReadOnly => vec![ExecutionPermission::ReadWorkspace],
            Self::WorkspaceWrite => vec![
                ExecutionPermission::ReadWorkspace,
                ExecutionPermission::WriteWorkspace,
                ExecutionPermission::RunWorkspaceCommand,
            ],
            Self::DangerFullAccess => ExecutionPermission::ALL.to_vec(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionApprovalPolicy {
    Never,
    OnFailure,
    OnRequest,
}

impl ExecutionApprovalPolicy {
    pub const ALL: [Self; 3] = [Self::Never, Self::OnFailure, Self::OnRequest];

    pub fn label(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::OnFailure => "on_failure",
            Self::OnRequest => "on_request",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPermission {
    ReadWorkspace,
    WriteWorkspace,
    RunWorkspaceCommand,
    AccessNetwork,
    AccessCredentials,
}

impl ExecutionPermission {
    pub const ALL: [Self; 5] = [
        Self::ReadWorkspace,
        Self::WriteWorkspace,
        Self::RunWorkspaceCommand,
        Self::AccessNetwork,
        Self::AccessCredentials,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::ReadWorkspace => "read_workspace",
            Self::WriteWorkspace => "write_workspace",
            Self::RunWorkspaceCommand => "run_workspace_command",
            Self::AccessNetwork => "access_network",
            Self::AccessCredentials => "access_credentials",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPermissionReuseScope {
    Turn,
    CommandPrefix,
    Hand,
}

impl ExecutionPermissionReuseScope {
    pub const ALL: [Self; 3] = [Self::Turn, Self::CommandPrefix, Self::Hand];

    pub fn label(self) -> &'static str {
        match self {
            Self::Turn => "turn",
            Self::CommandPrefix => "command_prefix",
            Self::Hand => "hand",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPermissionRequirement {
    pub summary: String,
    pub permissions: Vec<ExecutionPermission>,
}

impl ExecutionPermissionRequirement {
    pub fn new(summary: impl Into<String>, permissions: Vec<ExecutionPermission>) -> Self {
        Self {
            summary: summary.into(),
            permissions: canonicalize_permissions(permissions),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionEscalationRequest {
    pub justification: String,
    pub requested_permissions: Vec<ExecutionPermission>,
    pub reuse_scope: Option<ExecutionPermissionReuseScope>,
    pub command_prefix: Option<Vec<String>>,
}

impl ExecutionEscalationRequest {
    pub fn new(
        justification: impl Into<String>,
        requested_permissions: Vec<ExecutionPermission>,
        reuse_scope: Option<ExecutionPermissionReuseScope>,
        command_prefix: Option<Vec<String>>,
    ) -> Self {
        Self {
            justification: justification.into(),
            requested_permissions: canonicalize_permissions(requested_permissions),
            reuse_scope,
            command_prefix,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionGovernanceOutcomeKind {
    Allowed,
    Denied,
    EscalationRequired,
    PolicyUnavailable,
}

impl ExecutionGovernanceOutcomeKind {
    pub const ALL: [Self; 4] = [
        Self::Allowed,
        Self::Denied,
        Self::EscalationRequired,
        Self::PolicyUnavailable,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Denied => "denied",
            Self::EscalationRequired => "escalation_required",
            Self::PolicyUnavailable => "policy_unavailable",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionGovernanceOutcome {
    pub kind: ExecutionGovernanceOutcomeKind,
    pub reason: String,
    pub requirement: ExecutionPermissionRequirement,
    pub granted_permissions: Vec<ExecutionPermission>,
    pub escalation_request: Option<ExecutionEscalationRequest>,
}

impl ExecutionGovernanceOutcome {
    pub fn allowed(
        reason: impl Into<String>,
        requirement: ExecutionPermissionRequirement,
        granted_permissions: Vec<ExecutionPermission>,
    ) -> Self {
        Self {
            kind: ExecutionGovernanceOutcomeKind::Allowed,
            reason: reason.into(),
            requirement,
            granted_permissions: canonicalize_permissions(granted_permissions),
            escalation_request: None,
        }
    }

    pub fn denied(reason: impl Into<String>, requirement: ExecutionPermissionRequirement) -> Self {
        Self {
            kind: ExecutionGovernanceOutcomeKind::Denied,
            reason: reason.into(),
            requirement,
            granted_permissions: Vec::new(),
            escalation_request: None,
        }
    }

    pub fn escalation_required(
        reason: impl Into<String>,
        requirement: ExecutionPermissionRequirement,
        escalation_request: ExecutionEscalationRequest,
    ) -> Self {
        Self {
            kind: ExecutionGovernanceOutcomeKind::EscalationRequired,
            reason: reason.into(),
            requirement,
            granted_permissions: Vec::new(),
            escalation_request: Some(escalation_request),
        }
    }

    pub fn policy_unavailable(
        reason: impl Into<String>,
        requirement: ExecutionPermissionRequirement,
    ) -> Self {
        Self {
            kind: ExecutionGovernanceOutcomeKind::PolicyUnavailable,
            reason: reason.into(),
            requirement,
            granted_permissions: Vec::new(),
            escalation_request: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionGovernanceProfile {
    pub sandbox_mode: ExecutionSandboxMode,
    pub approval_policy: ExecutionApprovalPolicy,
    pub allowed_permissions: Vec<ExecutionPermission>,
    pub supported_reuse_scopes: Vec<ExecutionPermissionReuseScope>,
    pub downgrade_reason: Option<String>,
}

impl ExecutionGovernanceProfile {
    pub fn new(
        sandbox_mode: ExecutionSandboxMode,
        approval_policy: ExecutionApprovalPolicy,
        supported_reuse_scopes: Vec<ExecutionPermissionReuseScope>,
        downgrade_reason: Option<String>,
    ) -> Self {
        Self {
            sandbox_mode,
            approval_policy,
            allowed_permissions: sandbox_mode.default_permissions(),
            supported_reuse_scopes: canonicalize_reuse_scopes(supported_reuse_scopes),
            downgrade_reason,
        }
    }

    pub fn permits(&self, permission: ExecutionPermission) -> bool {
        self.allowed_permissions.contains(&permission)
    }

    pub fn supports_reuse_scope(&self, scope: ExecutionPermissionReuseScope) -> bool {
        self.supported_reuse_scopes.contains(&scope)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionHandKind {
    WorkspaceEditor,
    TerminalRunner,
    TransportMediator,
}

impl ExecutionHandKind {
    pub const ALL: [Self; 3] = [
        Self::WorkspaceEditor,
        Self::TerminalRunner,
        Self::TransportMediator,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::WorkspaceEditor => "workspace_editor",
            Self::TerminalRunner => "terminal_runner",
            Self::TransportMediator => "transport_mediator",
        }
    }

    pub fn default_authority(self) -> ExecutionHandAuthority {
        match self {
            Self::WorkspaceEditor | Self::TerminalRunner => ExecutionHandAuthority::WorkspaceScoped,
            Self::TransportMediator => ExecutionHandAuthority::CredentialMediated,
        }
    }

    pub fn default_summary(self) -> &'static str {
        match self {
            Self::WorkspaceEditor => "authored workspace mutation boundary",
            Self::TerminalRunner => "background shell execution boundary",
            Self::TransportMediator => "credential-bearing transport and tool mediation boundary",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionHandAuthority {
    WorkspaceScoped,
    CredentialMediated,
}

impl ExecutionHandAuthority {
    pub fn label(self) -> &'static str {
        match self {
            Self::WorkspaceScoped => "workspace_scoped",
            Self::CredentialMediated => "credential_mediated",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionHandOperation {
    Describe,
    Provision,
    Execute,
    Recover,
    Degrade,
}

impl ExecutionHandOperation {
    pub fn label(self) -> &'static str {
        match self {
            Self::Describe => "describe",
            Self::Provision => "provision",
            Self::Execute => "execute",
            Self::Recover => "recover",
            Self::Degrade => "degrade",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionHandPhase {
    Described,
    Provisioning,
    Ready,
    Executing,
    Recovering,
    Degraded,
    Failed,
}

impl ExecutionHandPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Described => "described",
            Self::Provisioning => "provisioning",
            Self::Ready => "ready",
            Self::Executing => "executing",
            Self::Recovering => "recovering",
            Self::Degraded => "degraded",
            Self::Failed => "failed",
        }
    }

    pub fn is_available(self) -> bool {
        matches!(
            self,
            Self::Described | Self::Provisioning | Self::Ready | Self::Executing | Self::Recovering
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionHandDescriptor {
    pub hand: ExecutionHandKind,
    pub authority: ExecutionHandAuthority,
    pub summary: String,
    pub supported_operations: Vec<ExecutionHandOperation>,
}

impl ExecutionHandDescriptor {
    pub fn new(
        hand: ExecutionHandKind,
        authority: ExecutionHandAuthority,
        summary: impl Into<String>,
        supported_operations: Vec<ExecutionHandOperation>,
    ) -> Self {
        Self {
            hand,
            authority,
            summary: summary.into(),
            supported_operations,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionHandDiagnostic {
    pub hand: ExecutionHandKind,
    pub phase: ExecutionHandPhase,
    pub authority: ExecutionHandAuthority,
    pub supported_operations: Vec<ExecutionHandOperation>,
    pub last_operation: Option<ExecutionHandOperation>,
    pub summary: String,
    pub last_error: Option<String>,
}

impl ExecutionHandDiagnostic {
    pub fn from_descriptor(descriptor: &ExecutionHandDescriptor) -> Self {
        Self {
            hand: descriptor.hand,
            phase: ExecutionHandPhase::Described,
            authority: descriptor.authority,
            supported_operations: descriptor.supported_operations.clone(),
            last_operation: Some(ExecutionHandOperation::Describe),
            summary: descriptor.summary.clone(),
            last_error: None,
        }
    }
}

pub fn default_local_execution_hand_descriptors() -> Vec<ExecutionHandDescriptor> {
    let supported_operations = vec![
        ExecutionHandOperation::Describe,
        ExecutionHandOperation::Provision,
        ExecutionHandOperation::Execute,
        ExecutionHandOperation::Recover,
        ExecutionHandOperation::Degrade,
    ];

    ExecutionHandKind::ALL
        .into_iter()
        .map(|hand| {
            ExecutionHandDescriptor::new(
                hand,
                hand.default_authority(),
                hand.default_summary(),
                supported_operations.clone(),
            )
        })
        .collect()
}

fn canonicalize_permissions(mut permissions: Vec<ExecutionPermission>) -> Vec<ExecutionPermission> {
    permissions.sort_unstable();
    permissions.dedup();
    permissions
}

fn canonicalize_reuse_scopes(
    mut scopes: Vec<ExecutionPermissionReuseScope>,
) -> Vec<ExecutionPermissionReuseScope> {
    scopes.sort_unstable();
    scopes.dedup();
    scopes
}

#[cfg(test)]
mod tests {
    use super::{
        ExecutionApprovalPolicy, ExecutionEscalationRequest, ExecutionGovernanceOutcome,
        ExecutionGovernanceOutcomeKind, ExecutionHandAuthority, ExecutionHandKind,
        ExecutionHandOperation, ExecutionHandPhase, ExecutionPermission,
        ExecutionPermissionRequirement, ExecutionPermissionReuseScope, ExecutionSandboxMode,
        default_local_execution_hand_descriptors,
    };

    #[test]
    fn execution_hand_contract_exposes_stable_local_runtime_vocabulary() {
        assert_eq!(
            ExecutionHandKind::ALL.map(ExecutionHandKind::label),
            ["workspace_editor", "terminal_runner", "transport_mediator"]
        );
        assert_eq!(
            [
                ExecutionHandOperation::Describe,
                ExecutionHandOperation::Provision,
                ExecutionHandOperation::Execute,
                ExecutionHandOperation::Recover,
                ExecutionHandOperation::Degrade,
            ]
            .map(ExecutionHandOperation::label),
            ["describe", "provision", "execute", "recover", "degrade"]
        );
        assert_eq!(
            [
                ExecutionHandPhase::Described,
                ExecutionHandPhase::Provisioning,
                ExecutionHandPhase::Ready,
                ExecutionHandPhase::Executing,
                ExecutionHandPhase::Recovering,
                ExecutionHandPhase::Degraded,
                ExecutionHandPhase::Failed,
            ]
            .map(ExecutionHandPhase::label),
            [
                "described",
                "provisioning",
                "ready",
                "executing",
                "recovering",
                "degraded",
                "failed",
            ]
        );

        let descriptors = default_local_execution_hand_descriptors();
        assert_eq!(descriptors.len(), 3);
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.hand == ExecutionHandKind::WorkspaceEditor
                && descriptor.authority == ExecutionHandAuthority::WorkspaceScoped
                && descriptor
                    .supported_operations
                    .contains(&ExecutionHandOperation::Execute)
        }));
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.hand == ExecutionHandKind::TransportMediator
                && descriptor.authority == ExecutionHandAuthority::CredentialMediated
                && descriptor
                    .supported_operations
                    .contains(&ExecutionHandOperation::Recover)
        }));
    }

    #[test]
    fn execution_governance_contract_exposes_stable_runtime_vocabulary() {
        assert_eq!(
            [
                ExecutionSandboxMode::ReadOnly,
                ExecutionSandboxMode::WorkspaceWrite,
                ExecutionSandboxMode::DangerFullAccess,
            ]
            .map(ExecutionSandboxMode::label),
            ["read_only", "workspace_write", "danger_full_access"]
        );
        assert_eq!(
            [
                ExecutionApprovalPolicy::Never,
                ExecutionApprovalPolicy::OnFailure,
                ExecutionApprovalPolicy::OnRequest,
            ]
            .map(ExecutionApprovalPolicy::label),
            ["never", "on_failure", "on_request"]
        );
        assert_eq!(
            [
                ExecutionPermission::ReadWorkspace,
                ExecutionPermission::WriteWorkspace,
                ExecutionPermission::RunWorkspaceCommand,
                ExecutionPermission::AccessNetwork,
                ExecutionPermission::AccessCredentials,
            ]
            .map(ExecutionPermission::label),
            [
                "read_workspace",
                "write_workspace",
                "run_workspace_command",
                "access_network",
                "access_credentials",
            ]
        );
        assert_eq!(
            [
                ExecutionPermissionReuseScope::Turn,
                ExecutionPermissionReuseScope::CommandPrefix,
                ExecutionPermissionReuseScope::Hand,
            ]
            .map(ExecutionPermissionReuseScope::label),
            ["turn", "command_prefix", "hand"]
        );

        let escalation = ExecutionEscalationRequest::new(
            "Need to run the repository test command",
            vec![ExecutionPermission::RunWorkspaceCommand],
            Some(ExecutionPermissionReuseScope::CommandPrefix),
            Some(vec!["cargo".to_string(), "test".to_string()]),
        );
        assert_eq!(
            escalation.justification,
            "Need to run the repository test command"
        );
        assert_eq!(
            escalation.requested_permissions,
            vec![ExecutionPermission::RunWorkspaceCommand]
        );
        assert_eq!(
            escalation.reuse_scope,
            Some(ExecutionPermissionReuseScope::CommandPrefix)
        );

        let requirement = ExecutionPermissionRequirement::new(
            "apply a workspace patch",
            vec![
                ExecutionPermission::ReadWorkspace,
                ExecutionPermission::WriteWorkspace,
            ],
        );
        assert_eq!(requirement.summary, "apply a workspace patch");
        assert_eq!(
            requirement.permissions,
            vec![
                ExecutionPermission::ReadWorkspace,
                ExecutionPermission::WriteWorkspace,
            ]
        );

        let denied = ExecutionGovernanceOutcome::denied(
            "write permission is not available under the active sandbox",
            requirement.clone(),
        );
        assert_eq!(denied.kind, ExecutionGovernanceOutcomeKind::Denied);
        assert_eq!(
            denied.reason,
            "write permission is not available under the active sandbox"
        );
        assert_eq!(denied.requirement, requirement);
        assert_eq!(denied.escalation_request, None);
    }
}
