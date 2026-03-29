use crate::domain::model::{
    ConversationThread, ConversationThreadRef, ConversationThreadStatus, TaskTraceId,
    ThreadCandidate, ThreadCandidateId, ThreadDecision, ThreadDecisionKind, ThreadMergeMode,
    TraceArtifactId, TraceBranchId, TraceRecordId, TurnTraceId,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ConversationSession {
    state: Arc<Mutex<ConversationSessionState>>,
}

#[derive(Debug)]
pub(crate) struct ConversationSessionState {
    pub task_id: TaskTraceId,
    pub next_turn_sequence: u64,
    pub next_candidate_sequence: u64,
    pub next_record_sequence: u64,
    pub next_artifact_sequence: u64,
    pub next_branch_sequence: u64,
    pub root_started: bool,
    pub root_last_record_id: Option<TraceRecordId>,
    pub branch_last_record_ids: HashMap<TraceBranchId, TraceRecordId>,
    pub recorder_warning_emitted: bool,
    pub active_thread: ConversationThreadRef,
    pub threads: HashMap<String, ConversationThread>,
    pub recent_thread_summaries: HashMap<String, String>,
}

impl ConversationSession {
    pub fn new(task_id: TaskTraceId) -> Self {
        let mainline = ConversationThread {
            thread_ref: ConversationThreadRef::Mainline,
            label: "mainline".to_string(),
            parent: None,
            status: ConversationThreadStatus::Active,
        };
        let mut threads = HashMap::new();
        threads.insert(mainline.thread_ref.stable_id(), mainline);

        Self {
            state: Arc::new(Mutex::new(ConversationSessionState {
                task_id,
                next_turn_sequence: 1,
                next_candidate_sequence: 1,
                next_record_sequence: 1,
                next_artifact_sequence: 1,
                next_branch_sequence: 1,
                root_started: false,
                root_last_record_id: None,
                branch_last_record_ids: HashMap::new(),
                recorder_warning_emitted: false,
                active_thread: ConversationThreadRef::Mainline,
                threads,
                recent_thread_summaries: HashMap::new(),
            })),
        }
    }

    pub(crate) fn state(&self) -> Arc<Mutex<ConversationSessionState>> {
        Arc::clone(&self.state)
    }

    pub fn task_id(&self) -> TaskTraceId {
        self.state
            .lock()
            .expect("conversation session lock")
            .task_id
            .clone()
    }

    pub fn allocate_turn_id(&self) -> TurnTraceId {
        let mut state = self.state.lock().expect("conversation session lock");
        let turn_id = TurnTraceId::new(format!(
            "{}.turn-{:04}",
            state.task_id.as_str(),
            state.next_turn_sequence
        ))
        .expect("generated turn id");
        state.next_turn_sequence += 1;
        turn_id
    }

    pub fn capture_candidate(&self, prompt: impl Into<String>) -> ThreadCandidate {
        let mut state = self.state.lock().expect("conversation session lock");
        let candidate_id = ThreadCandidateId::new(format!(
            "{}.candidate-{:04}",
            state.task_id.as_str(),
            state.next_candidate_sequence
        ))
        .expect("generated candidate id");
        let captured_sequence = state.next_candidate_sequence;
        state.next_candidate_sequence += 1;
        ThreadCandidate {
            candidate_id,
            prompt: prompt.into(),
            captured_from_turn_id: None,
            active_thread: state.active_thread.clone(),
            captured_sequence,
        }
    }

    pub fn active_thread(&self) -> ConversationThread {
        let state = self.state.lock().expect("conversation session lock");
        state
            .threads
            .get(&state.active_thread.stable_id())
            .cloned()
            .expect("active thread must exist")
    }

    pub fn known_threads(&self) -> Vec<ConversationThread> {
        let state = self.state.lock().expect("conversation session lock");
        let mut threads = state.threads.values().cloned().collect::<Vec<_>>();
        threads.sort_by_key(|thread| thread.thread_ref.stable_id());
        threads
    }

    pub fn recent_thread_summary(&self, thread_ref: &ConversationThreadRef) -> Option<String> {
        self.state
            .lock()
            .expect("conversation session lock")
            .recent_thread_summaries
            .get(&thread_ref.stable_id())
            .cloned()
    }

    pub fn note_thread_reply(&self, thread_ref: &ConversationThreadRef, prompt: &str, reply: &str) {
        let summary = format!(
            "prompt: {} | reply: {}",
            trim_summary(prompt),
            trim_summary(reply)
        );
        self.state
            .lock()
            .expect("conversation session lock")
            .recent_thread_summaries
            .insert(thread_ref.stable_id(), summary);
    }

    pub fn apply_thread_decision(
        &self,
        decision: &ThreadDecision,
        branch_id: Option<TraceBranchId>,
        candidate_prompt: &str,
    ) -> ConversationThreadRef {
        let mut state = self.state.lock().expect("conversation session lock");
        match decision.kind {
            ThreadDecisionKind::ContinueCurrent => {
                state.active_thread = decision.target_thread.clone();
            }
            ThreadDecisionKind::OpenChildThread => {
                let branch_id = branch_id.expect("child-thread decisions must allocate a branch");
                let current_thread_id = state.active_thread.stable_id();
                if let Some(current) = state.threads.get_mut(&current_thread_id) {
                    current.status = ConversationThreadStatus::Waiting;
                }
                let thread_ref = ConversationThreadRef::Branch(branch_id);
                let thread = ConversationThread {
                    thread_ref: thread_ref.clone(),
                    label: decision
                        .new_thread_label
                        .clone()
                        .unwrap_or_else(|| trim_summary(candidate_prompt)),
                    parent: Some(decision.target_thread.clone()),
                    status: ConversationThreadStatus::Active,
                };
                state.threads.insert(thread_ref.stable_id(), thread);
                state.active_thread = thread_ref.clone();
            }
            ThreadDecisionKind::MergeIntoTarget => {
                let source_id = state.active_thread.stable_id();
                if let Some(source) = state.threads.get_mut(&source_id) {
                    source.status = ConversationThreadStatus::Merged;
                }
                if let Some(target) = state.threads.get_mut(&decision.target_thread.stable_id()) {
                    target.status = ConversationThreadStatus::Active;
                }
                state.active_thread = decision.target_thread.clone();
                let merge_note = format!(
                    "{} merge into {} via {}",
                    source_id,
                    decision.target_thread.stable_id(),
                    decision
                        .merge_mode
                        .unwrap_or(ThreadMergeMode::Summary)
                        .label()
                );
                state
                    .recent_thread_summaries
                    .insert(decision.target_thread.stable_id(), merge_note);
            }
        }

        state.active_thread.clone()
    }

    pub fn branch_id_for_active_thread(&self) -> Option<TraceBranchId> {
        self.active_thread().thread_ref.branch_id()
    }

    pub fn next_branch_id(&self) -> TraceBranchId {
        let mut state = self.state.lock().expect("conversation session lock");
        let branch_id = TraceBranchId::new(format!(
            "{}.thread-{:04}",
            state.task_id.as_str(),
            state.next_branch_sequence
        ))
        .expect("generated branch id");
        state.next_branch_sequence += 1;
        branch_id
    }

    pub(crate) fn next_artifact_id(&self, turn_id: &TurnTraceId) -> TraceArtifactId {
        let mut state = self.state.lock().expect("conversation session lock");
        let artifact_id = TraceArtifactId::new(format!(
            "{}.artifact-{:04}",
            turn_id.as_str(),
            state.next_artifact_sequence
        ))
        .expect("generated artifact id");
        state.next_artifact_sequence += 1;
        artifact_id
    }
}

fn trim_summary(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.chars().count() <= 80 {
        return trimmed.to_string();
    }
    let prefix = trimmed.chars().take(80).collect::<String>();
    format!("{}...", prefix.trim_end())
}

#[cfg(test)]
mod tests {
    use super::ConversationSession;
    use crate::domain::model::{
        ConversationThreadRef, TaskTraceId, ThreadDecision, ThreadDecisionId, ThreadDecisionKind,
    };

    #[test]
    fn new_session_starts_on_mainline() {
        let session = ConversationSession::new(TaskTraceId::new("task-1").expect("task"));

        assert_eq!(
            session.active_thread().thread_ref,
            ConversationThreadRef::Mainline
        );
        assert_eq!(session.known_threads().len(), 1);
    }

    #[test]
    fn child_thread_decisions_activate_new_branch() {
        let session = ConversationSession::new(TaskTraceId::new("task-2").expect("task"));
        let branch_id = session.next_branch_id();

        let active = session.apply_thread_decision(
            &ThreadDecision {
                decision_id: ThreadDecisionId::new("decision-1").expect("decision"),
                candidate_id: session.capture_candidate("Investigate it").candidate_id,
                kind: ThreadDecisionKind::OpenChildThread,
                rationale: "split off a child thread".to_string(),
                target_thread: ConversationThreadRef::Mainline,
                new_thread_label: Some("investigate".to_string()),
                merge_mode: None,
                merge_summary: None,
            },
            Some(branch_id.clone()),
            "Investigate it",
        );

        assert_eq!(active, ConversationThreadRef::Branch(branch_id));
        assert_eq!(session.known_threads().len(), 2);
    }
}
