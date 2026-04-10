use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::{
        ExecutionHandAuthority, ExecutionHandKind, ExecutionHandOperation, ExecutionHandPhase,
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
}
