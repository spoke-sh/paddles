use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalScenario {
    pub id: String,
    pub title: String,
    pub requires_network: bool,
    pub expected_contracts: Vec<String>,
}

impl EvalScenario {
    pub fn local(
        id: impl Into<String>,
        title: impl Into<String>,
        expected_contracts: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            requires_network: false,
            expected_contracts,
        }
    }

    pub fn networked(
        id: impl Into<String>,
        title: impl Into<String>,
        expected_contracts: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            requires_network: true,
            expected_contracts,
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
    pub contract: String,
    pub status: EvalStatus,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvalStatus {
    Passed,
    Failed,
}
