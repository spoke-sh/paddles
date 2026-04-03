use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessChamber {
    Idle,
    Interpretation,
    Routing,
    Planning,
    Gathering,
    Tooling,
    Threading,
    Generation,
    Rendering,
    Governor,
}

impl HarnessChamber {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Interpretation => "interpretation",
            Self::Routing => "routing",
            Self::Planning => "planning",
            Self::Gathering => "gathering",
            Self::Tooling => "tooling",
            Self::Threading => "threading",
            Self::Generation => "generation",
            Self::Rendering => "rendering",
            Self::Governor => "governor",
        }
    }
}

impl fmt::Display for HarnessChamber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessStatus {
    Active,
    Completed,
    Intervening,
}

impl HarnessStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Intervening => "intervening",
        }
    }
}

impl fmt::Display for HarnessStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeoutPhase {
    Nominal,
    Slow,
    Stalled,
    Expired,
}

impl TimeoutPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Nominal => "nominal",
            Self::Slow => "slow",
            Self::Stalled => "stalled",
            Self::Expired => "expired",
        }
    }
}

impl fmt::Display for TimeoutPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutState {
    pub phase: TimeoutPhase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline_seconds: Option<u64>,
}

impl Default for TimeoutState {
    fn default() -> Self {
        Self {
            phase: TimeoutPhase::Nominal,
            elapsed_seconds: None,
            deadline_seconds: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernorState {
    pub status: HarnessStatus,
    pub timeout: TimeoutState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intervention: Option<String>,
}

impl GovernorState {
    pub fn active() -> Self {
        Self {
            status: HarnessStatus::Active,
            timeout: TimeoutState::default(),
            intervention: None,
        }
    }

    pub fn intervening(intervention: impl Into<String>) -> Self {
        Self {
            status: HarnessStatus::Intervening,
            timeout: TimeoutState::default(),
            intervention: Some(intervention.into()),
        }
    }

    pub fn with_timeout(mut self, timeout: TimeoutState) -> Self {
        self.timeout = timeout;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessSnapshot {
    pub chamber: HarnessChamber,
    pub governor: GovernorState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl HarnessSnapshot {
    pub fn active(chamber: HarnessChamber) -> Self {
        Self {
            chamber,
            governor: GovernorState::active(),
            detail: None,
        }
    }

    pub fn intervening(chamber: HarnessChamber, intervention: impl Into<String>) -> Self {
        Self {
            chamber,
            governor: GovernorState::intervening(intervention),
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_governor(mut self, governor: GovernorState) -> Self {
        self.governor = governor;
        self
    }
}
