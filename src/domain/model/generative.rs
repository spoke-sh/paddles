use super::render::RenderDocument;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseMode {
    DirectAnswer,
    GroundedAnswer,
    CompletedEdit,
    BlockedEdit,
    PolicyRefusal,
}

impl ResponseMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::DirectAnswer => "direct_answer",
            Self::GroundedAnswer => "grounded_answer",
            Self::CompletedEdit => "completed_edit",
            Self::BlockedEdit => "blocked_edit",
            Self::PolicyRefusal => "policy_refusal",
        }
    }

    pub fn from_label(value: &str) -> Option<Self> {
        match value {
            "direct_answer" => Some(Self::DirectAnswer),
            "grounded_answer" => Some(Self::GroundedAnswer),
            "completed_edit" => Some(Self::CompletedEdit),
            "blocked_edit" => Some(Self::BlockedEdit),
            "policy_refusal" => Some(Self::PolicyRefusal),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoredResponse {
    pub mode: ResponseMode,
    pub document: RenderDocument,
}

impl AuthoredResponse {
    pub fn from_plain_text(mode: ResponseMode, text: &str) -> Self {
        Self {
            mode,
            document: RenderDocument::from_assistant_plain_text(text),
        }
    }

    pub fn to_plain_text(&self) -> String {
        self.document.to_plain_text()
    }
}
