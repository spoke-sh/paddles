use super::{TraceBranchStatus, TraceRecordKind, TraceReplay};
use paddles_conversation::{
    ConversationThread, ConversationThreadRef, ConversationThreadStatus, ThreadMergeRecord,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationReplayView {
    pub mainline_turns: usize,
    pub threads: Vec<ConversationThread>,
    pub merge_records: Vec<ThreadMergeRecord>,
}

impl ConversationReplayView {
    pub fn from_trace_replay(replay: &TraceReplay) -> Self {
        let mut threads = BTreeMap::new();
        threads.insert(
            "mainline".to_string(),
            ConversationThread {
                thread_ref: ConversationThreadRef::Mainline,
                label: "mainline".to_string(),
                parent: None,
                status: ConversationThreadStatus::Active,
            },
        );
        let mut mainline_turns = 0;
        let mut merge_records = Vec::new();

        for record in &replay.records {
            match &record.kind {
                TraceRecordKind::TaskRootStarted(_) | TraceRecordKind::TurnStarted(_) => {
                    if let Some(branch_id) = &record.lineage.branch_id {
                        threads
                            .entry(branch_id.as_str().to_string())
                            .or_insert_with(|| ConversationThread {
                                thread_ref: ConversationThreadRef::Branch(branch_id.clone()),
                                label: branch_id.as_str().to_string(),
                                parent: None,
                                status: ConversationThreadStatus::Waiting,
                            });
                    } else {
                        mainline_turns += 1;
                    }
                }
                TraceRecordKind::PlannerBranchDeclared(branch) => {
                    threads.insert(
                        branch.branch_id.as_str().to_string(),
                        ConversationThread {
                            thread_ref: ConversationThreadRef::Branch(branch.branch_id.clone()),
                            label: branch.label.clone(),
                            parent: branch
                                .parent_branch_id
                                .as_ref()
                                .map(|parent| ConversationThreadRef::Branch(parent.clone())),
                            status: match branch.status {
                                TraceBranchStatus::Merged => ConversationThreadStatus::Merged,
                                TraceBranchStatus::Active => ConversationThreadStatus::Active,
                                _ => ConversationThreadStatus::Waiting,
                            },
                        },
                    );
                }
                TraceRecordKind::ThreadMerged(merge) => {
                    if let Some(source) = threads.get_mut(&merge.source_thread.stable_id()) {
                        source.status = ConversationThreadStatus::Merged;
                    }
                    if let Some(target) = threads.get_mut(&merge.target_thread.stable_id()) {
                        target.status = ConversationThreadStatus::Active;
                    }
                    merge_records.push(merge.clone());
                }
                _ => {}
            }
        }

        Self {
            mainline_turns,
            threads: threads.into_values().collect(),
            merge_records,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ConversationReplayView, ConversationThread, ConversationThreadRef, ConversationThreadStatus,
    };
    use crate::domain::model::{
        ArtifactEnvelope, ArtifactKind, TaskTraceId, ThreadCandidateId, ThreadDecisionId,
        ThreadDecisionKind, ThreadMergeMode, TraceArtifactId, TraceBranch, TraceBranchId,
        TraceBranchStatus, TraceLineage, TraceRecord, TraceRecordId, TraceRecordKind, TraceReplay,
        TraceTaskRoot, TurnTraceId,
    };

    #[test]
    fn mainline_thread_has_stable_identity() {
        let thread = ConversationThreadRef::Mainline;

        assert_eq!(thread.stable_id(), "mainline");
        assert_eq!(thread.label(), "mainline");
        assert_eq!(thread.branch_id(), None);
    }

    #[test]
    fn branch_thread_uses_branch_id_as_stable_identity() {
        let thread =
            ConversationThreadRef::Branch(TraceBranchId::new("thread-1").expect("branch id"));

        assert_eq!(thread.stable_id(), "thread-1");
        assert_eq!(thread.branch_id().expect("branch").as_str(), "thread-1");
    }

    #[test]
    fn thread_contract_types_render_generic_labels() {
        let thread = ConversationThread {
            thread_ref: ConversationThreadRef::Mainline,
            label: "mainline".to_string(),
            parent: None,
            status: ConversationThreadStatus::Active,
        };

        assert_eq!(thread.summary(), "mainline (active)");
        assert_eq!(
            ThreadDecisionKind::OpenChildThread.label(),
            "open-child-thread"
        );
        assert_eq!(ThreadMergeMode::Summary.label(), "summary");
        assert_eq!(
            ThreadCandidateId::new("candidate-1")
                .expect("candidate")
                .as_str(),
            "candidate-1"
        );
        assert_eq!(
            ThreadDecisionId::new("decision-1")
                .expect("decision")
                .as_str(),
            "decision-1"
        );
    }

    #[test]
    fn replay_view_reconstructs_threads_and_merges() {
        let replay = TraceReplay {
            task_id: TaskTraceId::new("task-1").expect("task"),
            records: vec![
                TraceRecord {
                    record_id: TraceRecordId::new("record-1").expect("record"),
                    sequence: 1,
                    lineage: TraceLineage {
                        task_id: TaskTraceId::new("task-1").expect("task"),
                        turn_id: TurnTraceId::new("turn-1").expect("turn"),
                        branch_id: None,
                        parent_record_id: None,
                    },
                    kind: TraceRecordKind::TaskRootStarted(TraceTaskRoot {
                        prompt: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-1").expect("artifact"),
                            ArtifactKind::Prompt,
                            "prompt",
                            "hello",
                            256,
                        ),
                        interpretation: None,
                        planner_model: "planner".to_string(),
                        synthesizer_model: "synth".to_string(),
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-2").expect("record"),
                    sequence: 2,
                    lineage: TraceLineage {
                        task_id: TaskTraceId::new("task-1").expect("task"),
                        turn_id: TurnTraceId::new("turn-2").expect("turn"),
                        branch_id: Some(TraceBranchId::new("thread-1").expect("branch")),
                        parent_record_id: Some(TraceRecordId::new("record-1").expect("record")),
                    },
                    kind: TraceRecordKind::PlannerBranchDeclared(TraceBranch {
                        branch_id: TraceBranchId::new("thread-1").expect("branch"),
                        label: "investigate".to_string(),
                        status: TraceBranchStatus::Pending,
                        rationale: None,
                        parent_branch_id: None,
                        created_from_record_id: Some(
                            TraceRecordId::new("record-1").expect("record"),
                        ),
                    }),
                },
            ],
        };

        let view = ConversationReplayView::from_trace_replay(&replay);

        assert_eq!(view.mainline_turns, 1);
        assert_eq!(view.threads.len(), 2);
        assert!(view.merge_records.is_empty());
    }
}
