use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvalHarnessContract {
    CapabilityDisclosure,
    RecursiveEvidence,
    ToolRecovery,
    EditObligation,
    Delegation,
    ContextPressure,
    Replay,
    ExternalCapability,
    OfflineGuard,
}

impl EvalHarnessContract {
    pub fn label(self) -> &'static str {
        match self {
            Self::CapabilityDisclosure => "capability-disclosure",
            Self::RecursiveEvidence => "recursive-evidence",
            Self::ToolRecovery => "tool-recovery",
            Self::EditObligation => "edit-obligation",
            Self::Delegation => "delegation",
            Self::ContextPressure => "context-pressure",
            Self::Replay => "replay",
            Self::ExternalCapability => "external-capability",
            Self::OfflineGuard => "offline-guard",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalScenario {
    pub id: String,
    pub title: String,
    pub requires_network: bool,
    pub expected_contracts: Vec<EvalHarnessContract>,
    pub observed_contracts: Vec<EvalHarnessContract>,
}

impl EvalScenario {
    pub fn local(
        id: impl Into<String>,
        title: impl Into<String>,
        expected_contracts: Vec<EvalHarnessContract>,
    ) -> Self {
        let observed_contracts = expected_contracts.clone();
        Self {
            id: id.into(),
            title: title.into(),
            requires_network: false,
            expected_contracts,
            observed_contracts,
        }
    }

    pub fn networked(
        id: impl Into<String>,
        title: impl Into<String>,
        expected_contracts: Vec<EvalHarnessContract>,
    ) -> Self {
        let observed_contracts = expected_contracts.clone();
        Self {
            id: id.into(),
            title: title.into(),
            requires_network: true,
            expected_contracts,
            observed_contracts,
        }
    }

    pub fn local_with_observed_contracts(
        id: impl Into<String>,
        title: impl Into<String>,
        expected_contracts: Vec<EvalHarnessContract>,
        observed_contracts: Vec<EvalHarnessContract>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            requires_network: false,
            expected_contracts,
            observed_contracts,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalRunConfig {
    pub offline: bool,
}

impl Default for EvalRunConfig {
    fn default() -> Self {
        Self { offline: true }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalReport {
    pub scenario_id: String,
    pub status: EvalStatus,
    pub outcomes: Vec<EvalOutcome>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalOutcome {
    pub contract: EvalHarnessContract,
    pub status: EvalStatus,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvalStatus {
    Passed,
    Failed,
}
