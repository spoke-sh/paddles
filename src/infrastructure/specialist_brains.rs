use crate::domain::ports::{
    SpecialistBrain, SpecialistBrainCapability, SpecialistBrainNote, SpecialistBrainRequest,
};
use crate::infrastructure::harness_profile::HarnessProfileSelection;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct SpecialistBrainRegistry {
    brains: HashMap<String, Arc<dyn SpecialistBrain>>,
}

impl SpecialistBrainRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register(Arc::new(SessionContinuitySpecialistBrain));
        registry
    }

    pub fn register(&mut self, brain: Arc<dyn SpecialistBrain>) {
        self.brains.insert(brain.id().to_string(), brain);
    }

    pub fn runtime_notes(
        &self,
        profile: &HarnessProfileSelection,
        request: &SpecialistBrainRequest,
    ) -> Vec<String> {
        profile
            .active_specialist_brain_ids()
            .iter()
            .map(|brain_id| {
                let Some(brain) = self.brains.get(*brain_id) else {
                    return format!(
                        "Specialist brain [{brain_id}] unavailable: no registered implementation exists for the active harness."
                    );
                };
                match brain.capability(request) {
                    SpecialistBrainCapability::Available => match brain.runtime_note(request) {
                        Ok(note) => note.note,
                        Err(err) => {
                            format!("Specialist brain [{}] unavailable: {}", brain.id(), err)
                        }
                    },
                    SpecialistBrainCapability::Unsupported { reason } => {
                        format!("Specialist brain [{}] unavailable: {}", brain.id(), reason)
                    }
                }
            })
            .collect()
    }
}

struct SessionContinuitySpecialistBrain;

impl SpecialistBrain for SessionContinuitySpecialistBrain {
    fn id(&self) -> &'static str {
        "session-continuity-v1"
    }

    fn capability(&self, request: &SpecialistBrainRequest) -> SpecialistBrainCapability {
        if request.active_profile_id != "recursive-structured-v1" {
            return SpecialistBrainCapability::Unsupported {
                reason: format!(
                    "{} only activates for recursive-structured-v1; active profile is {}.",
                    self.id(),
                    request.active_profile_id
                ),
            };
        }

        if request.session_context.turn_summaries.is_empty() {
            return SpecialistBrainCapability::Unsupported {
                reason: "no durable session turns are available yet.".to_string(),
            };
        }

        SpecialistBrainCapability::Available
    }

    fn runtime_note(&self, request: &SpecialistBrainRequest) -> Result<SpecialistBrainNote> {
        Ok(SpecialistBrainNote {
            brain_id: self.id().to_string(),
            note: format!(
                "Specialist brain [{}] reviewed {} durable turn summary/summaries from the active session before recursive planning.",
                self.id(),
                request.session_context.turn_summaries.len()
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{
        ArtifactEnvelope, ArtifactKind, TraceCheckpointKind, TraceCompletionCheckpoint,
        TraceHarnessProfileSelection, TraceLineage, TraceRecord, TraceRecordKind, TraceReplay,
        TraceTaskRoot,
    };
    use crate::domain::ports::{
        TraceSessionContextQuery, TraceSessionContextSlice, TraceSessionWake,
    };
    use crate::infrastructure::providers::ModelProvider;
    use paddles_conversation::{
        TaskTraceId, TraceArtifactId, TraceCheckpointId, TraceRecordId, TurnTraceId,
    };
    use std::path::PathBuf;

    #[test]
    fn session_continuity_specialist_adds_runtime_note_for_structured_profiles() {
        let registry = SpecialistBrainRegistry::new();
        let profile = HarnessProfileSelection::resolve(
            &ModelProvider::Openai.capability_surface("gpt-5.4"),
            &ModelProvider::Openai.capability_surface("gpt-5.4"),
        );

        let notes = registry.runtime_notes(
            &profile,
            &SpecialistBrainRequest {
                user_prompt: "summarize session".to_string(),
                workspace_root: PathBuf::from("/workspace"),
                active_profile_id: profile.active_profile_id().to_string(),
                session_context: sample_session_context_slice(),
            },
        );

        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("session-continuity-v1"));
        assert!(notes[0].contains("1 durable turn summary"));
    }

    #[test]
    fn session_continuity_specialist_reports_clear_profile_fallback() {
        let registry = SpecialistBrainRegistry::new();
        let profile = HarnessProfileSelection::resolve(
            &ModelProvider::Anthropic.capability_surface("claude-sonnet-4-20250514"),
            &ModelProvider::Sift.capability_surface("qwen-1.5b"),
        );

        let notes = registry.runtime_notes(
            &profile,
            &SpecialistBrainRequest {
                user_prompt: "summarize session".to_string(),
                workspace_root: PathBuf::from("/workspace"),
                active_profile_id: profile.active_profile_id().to_string(),
                session_context: sample_session_context_slice(),
            },
        );

        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("unavailable"));
        assert!(notes[0].contains("prompt-envelope-safe-v1"));
    }

    fn sample_session_context_slice() -> TraceSessionContextSlice {
        let task_id = TaskTraceId::new("task-specialist").expect("task");
        let turn_id = TurnTraceId::new("task-specialist.turn-0001").expect("turn");
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
                    kind: TraceRecordKind::TaskRootStarted(TraceTaskRoot {
                        prompt: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-1").expect("artifact"),
                            ArtifactKind::Prompt,
                            "prompt",
                            "hello",
                            200,
                        ),
                        interpretation: None,
                        planner_model: "planner".to_string(),
                        synthesizer_model: "synth".to_string(),
                        harness_profile: TraceHarnessProfileSelection {
                            requested_profile_id: "recursive-structured-v1".to_string(),
                            active_profile_id: "recursive-structured-v1".to_string(),
                            downgrade_reason: None,
                        },
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-2").expect("record"),
                    sequence: 2,
                    lineage: TraceLineage {
                        task_id,
                        turn_id,
                        branch_id: None,
                        parent_record_id: Some(
                            TraceRecordId::new("record-1").expect("parent record"),
                        ),
                    },
                    kind: TraceRecordKind::CompletionCheckpoint(TraceCompletionCheckpoint {
                        checkpoint_id: TraceCheckpointId::new("checkpoint-1").expect("checkpoint"),
                        kind: TraceCheckpointKind::TurnCompleted,
                        summary: "done".to_string(),
                        response: Some(ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-2").expect("artifact"),
                            ArtifactKind::ModelOutput,
                            "reply",
                            "hi",
                            200,
                        )),
                        authored_response: None,
                        citations: Vec::new(),
                        grounded: true,
                    }),
                },
            ],
        };

        TraceSessionContextSlice::from_replay(
            &replay,
            &TraceSessionWake::from_replay(&replay),
            &TraceSessionContextQuery::AdaptiveReplay { turn_limit: 4 },
        )
        .expect("session context slice")
    }
}
