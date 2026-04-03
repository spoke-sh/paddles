use super::interpretation::WorkspaceAction;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionFrame {
    pub primary_intent: InstructionIntent,
    pub obligations: Vec<InstructionObligation>,
    #[serde(default)]
    pub candidate_files: Vec<String>,
}

impl InstructionFrame {
    pub fn for_edit(candidate_files: Vec<String>) -> Self {
        Self {
            primary_intent: InstructionIntent::Edit,
            obligations: vec![InstructionObligation {
                deliverable: InstructionDeliverable::AppliedEdit,
                satisfaction: InstructionSatisfaction::Pending,
            }],
            candidate_files,
        }
    }

    pub fn requires_applied_edit(&self) -> bool {
        self.obligations.iter().any(|obligation| {
            obligation.deliverable == InstructionDeliverable::AppliedEdit
                && obligation.satisfaction != InstructionSatisfaction::Satisfied
        })
    }

    pub fn note_successful_workspace_action(&mut self, action: &WorkspaceAction) {
        if matches!(
            action,
            WorkspaceAction::WriteFile { .. }
                | WorkspaceAction::ReplaceInFile { .. }
                | WorkspaceAction::ApplyPatch { .. }
        ) {
            for obligation in &mut self.obligations {
                if obligation.deliverable == InstructionDeliverable::AppliedEdit {
                    obligation.satisfaction = InstructionSatisfaction::Satisfied;
                }
            }
        }
    }

    pub fn candidate_summary(&self) -> Option<String> {
        if self.candidate_files.is_empty() {
            None
        } else {
            Some(self.candidate_files.join(", "))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionIntent {
    Answer,
    Edit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionObligation {
    pub deliverable: InstructionDeliverable,
    pub satisfaction: InstructionSatisfaction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionDeliverable {
    AppliedEdit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionSatisfaction {
    Pending,
    Satisfied,
    Blocked,
}
