use super::interpretation::WorkspaceAction;
use crate::domain::ports::EntityResolutionOutcome;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionFrame {
    pub primary_intent: InstructionIntent,
    pub obligations: Vec<InstructionObligation>,
    #[serde(default)]
    pub candidate_files: Vec<String>,
    #[serde(default)]
    pub resolution: Option<EntityResolutionOutcome>,
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
            resolution: None,
        }
    }

    pub fn for_commit() -> Self {
        Self {
            primary_intent: InstructionIntent::Commit,
            obligations: vec![InstructionObligation {
                deliverable: InstructionDeliverable::AppliedCommit,
                satisfaction: InstructionSatisfaction::Pending,
            }],
            candidate_files: Vec::new(),
            resolution: None,
        }
    }

    pub fn requires_applied_edit(&self) -> bool {
        self.obligations.iter().any(|obligation| {
            obligation.deliverable == InstructionDeliverable::AppliedEdit
                && obligation.satisfaction != InstructionSatisfaction::Satisfied
        })
    }

    pub fn requires_applied_commit(&self) -> bool {
        self.obligations.iter().any(|obligation| {
            obligation.deliverable == InstructionDeliverable::AppliedCommit
                && obligation.satisfaction != InstructionSatisfaction::Satisfied
        })
    }

    pub fn has_pending_workspace_obligation(&self) -> bool {
        self.requires_applied_edit() || self.requires_applied_commit()
    }

    pub fn ensure_applied_edit(&mut self, candidate_files: Vec<String>) {
        if !self
            .obligations
            .iter()
            .any(|obligation| obligation.deliverable == InstructionDeliverable::AppliedEdit)
        {
            self.obligations.push(InstructionObligation {
                deliverable: InstructionDeliverable::AppliedEdit,
                satisfaction: InstructionSatisfaction::Pending,
            });
        }
        if self.primary_intent == InstructionIntent::Answer {
            self.primary_intent = InstructionIntent::Edit;
        }
        for candidate in candidate_files {
            if !self.candidate_files.contains(&candidate) {
                self.candidate_files.push(candidate);
            }
        }
    }

    pub fn ensure_applied_commit(&mut self) {
        if !self
            .obligations
            .iter()
            .any(|obligation| obligation.deliverable == InstructionDeliverable::AppliedCommit)
        {
            self.obligations.push(InstructionObligation {
                deliverable: InstructionDeliverable::AppliedCommit,
                satisfaction: InstructionSatisfaction::Pending,
            });
        }
        if self.primary_intent == InstructionIntent::Answer {
            self.primary_intent = InstructionIntent::Commit;
        }
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

        if matches!(action, WorkspaceAction::Shell { command } if is_git_commit_command(command)) {
            for obligation in &mut self.obligations {
                if obligation.deliverable == InstructionDeliverable::AppliedCommit {
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

    pub fn note_resolution(&mut self, resolution: EntityResolutionOutcome) {
        self.resolution = Some(resolution);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionIntent {
    Answer,
    Edit,
    Commit,
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
    AppliedCommit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionSatisfaction {
    Pending,
    Satisfied,
    Blocked,
}

fn is_git_commit_command(command: &str) -> bool {
    let trimmed = command.trim();
    trimmed == "git commit" || trimmed.starts_with("git commit ")
}
