use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

macro_rules! paddles_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self> {
                let value = value.into();
                ensure!(
                    !value.trim().is_empty(),
                    concat!(stringify!($name), " must not be empty")
                );
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

paddles_id!(TaskTraceId);
paddles_id!(TurnTraceId);
paddles_id!(TraceRecordId);
paddles_id!(TraceBranchId);
paddles_id!(TraceArtifactId);
paddles_id!(TraceCheckpointId);
paddles_id!(ThreadCandidateId);
paddles_id!(ThreadDecisionId);
paddles_id!(TurnControlId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextTier {
    Inline,
    Transit,
    Sift,
    Filesystem,
}

impl ContextTier {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Transit => "transit",
            Self::Sift => "sift",
            Self::Filesystem => "filesystem",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextLocator {
    Inline {
        content: String,
    },
    Transit {
        task_id: TaskTraceId,
        record_id: TraceRecordId,
    },
    Sift {
        index_ref: String,
    },
    Filesystem {
        path: std::path::PathBuf,
    },
}

impl ContextLocator {
    pub fn tier(&self) -> ContextTier {
        match self {
            Self::Inline { .. } => ContextTier::Inline,
            Self::Transit { .. } => ContextTier::Transit,
            Self::Sift { .. } => ContextTier::Sift,
            Self::Filesystem { .. } => ContextTier::Filesystem,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactKind {
    Prompt,
    Interpretation,
    ModelOutput,
    ToolInvocation,
    ToolOutput,
    EvidenceBundle,
    PlannerTrace,
    Selection,
    Checkpoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactEnvelope {
    pub artifact_id: TraceArtifactId,
    pub kind: ArtifactKind,
    pub mime_type: String,
    pub summary: String,
    pub byte_count: usize,
    pub inline_content: Option<String>,
    #[serde(deserialize_with = "deserialize_locator", default)]
    pub locator: Option<ContextLocator>,
    pub truncated: bool,
    pub labels: std::collections::BTreeMap<String, String>,
}

fn deserialize_locator<'de, D>(deserializer: D) -> Result<Option<ContextLocator>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value: Option<serde_json::Value> = Deserialize::deserialize(deserializer)?;
    match value {
        None => Ok(None),
        Some(serde_json::Value::String(_s)) => {
            // Backward compatibility for old paddles-artifact:// IDs.
            // We can't resolve them yet without task_id, so treat as missing.
            Ok(None)
        }
        Some(v) => serde_json::from_value(v).map_err(Error::custom),
    }
}

impl ArtifactEnvelope {
    pub fn text(
        artifact_id: TraceArtifactId,
        kind: ArtifactKind,
        summary: impl Into<String>,
        content: impl Into<String>,
        inline_limit: usize,
    ) -> Self {
        let content = content.into();
        let char_count = content.chars().count();
        let truncated = char_count > inline_limit;
        let inline_content = if content.is_empty() {
            None
        } else if truncated {
            let prefix = content.chars().take(inline_limit).collect::<String>();
            Some(format!("{}...[truncated]", prefix.trim_end()))
        } else {
            Some(content.clone())
        };

        Self {
            artifact_id,
            kind,
            mime_type: "text/plain".to_string(),
            summary: summary.into(),
            byte_count: content.len(),
            inline_content,
            locator: None, // Will be set by recorder/orchestrator
            truncated,
            labels: std::collections::BTreeMap::new(),
        }
    }

    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = mime_type.into();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationThreadRef {
    Mainline,
    Branch(TraceBranchId),
}

impl ConversationThreadRef {
    pub fn stable_id(&self) -> String {
        match self {
            Self::Mainline => "mainline".to_string(),
            Self::Branch(branch_id) => branch_id.as_str().to_string(),
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Mainline => "mainline".to_string(),
            Self::Branch(branch_id) => format!("thread {}", branch_id.as_str()),
        }
    }

    pub fn branch_id(&self) -> Option<TraceBranchId> {
        match self {
            Self::Mainline => None,
            Self::Branch(branch_id) => Some(branch_id.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationThreadStatus {
    Active,
    Waiting,
    Merged,
}

impl ConversationThreadStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Waiting => "waiting",
            Self::Merged => "merged",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationThread {
    pub thread_ref: ConversationThreadRef,
    pub label: String,
    pub parent: Option<ConversationThreadRef>,
    pub status: ConversationThreadStatus,
}

impl ConversationThread {
    pub fn summary(&self) -> String {
        format!("{} ({})", self.label, self.status.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadCandidate {
    pub candidate_id: ThreadCandidateId,
    pub prompt: String,
    pub captured_from_turn_id: Option<TurnTraceId>,
    pub active_thread: ConversationThreadRef,
    pub captured_sequence: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadMergeMode {
    Backlink,
    Summary,
    Merge,
}

impl ThreadMergeMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Backlink => "backlink",
            Self::Summary => "summary",
            Self::Merge => "merge",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadDecisionKind {
    ContinueCurrent,
    OpenChildThread,
    MergeIntoTarget,
}

impl ThreadDecisionKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ContinueCurrent => "continue-current-thread",
            Self::OpenChildThread => "open-child-thread",
            Self::MergeIntoTarget => "merge-into-target",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadDecision {
    pub decision_id: ThreadDecisionId,
    pub candidate_id: ThreadCandidateId,
    pub kind: ThreadDecisionKind,
    pub rationale: String,
    pub target_thread: ConversationThreadRef,
    pub new_thread_label: Option<String>,
    pub merge_mode: Option<ThreadMergeMode>,
    pub merge_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadMergeRecord {
    pub decision: ThreadDecision,
    pub source_thread: ConversationThreadRef,
    pub target_thread: ConversationThreadRef,
    pub summary_artifact: Option<ArtifactEnvelope>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnControlKind {
    Steer,
    Interrupt,
}

impl TurnControlKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Steer => "steer",
            Self::Interrupt => "interrupt",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnControlRequest {
    pub control_id: TurnControlId,
    pub turn_id: TurnTraceId,
    pub active_thread: ConversationThreadRef,
    pub kind: TurnControlKind,
    pub prompt: Option<String>,
    pub captured_sequence: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnControlRequestError {
    NoActiveTurn,
}

#[derive(Clone)]
pub struct ConversationSession {
    state: Arc<Mutex<ConversationSessionState>>,
}

#[derive(Debug)]
pub struct ConversationSessionState {
    pub task_id: TaskTraceId,
    pub next_turn_sequence: u64,
    pub next_candidate_sequence: u64,
    pub next_exchange_sequence: u64,
    pub next_record_sequence: u64,
    pub next_artifact_sequence: u64,
    pub next_branch_sequence: u64,
    pub next_turn_control_sequence: u64,
    pub root_started: bool,
    pub root_last_record_id: Option<TraceRecordId>,
    pub branch_last_record_ids: HashMap<TraceBranchId, TraceRecordId>,
    pub recorder_warning_emitted: bool,
    pub active_turn_id: Option<TurnTraceId>,
    pub active_thread: ConversationThreadRef,
    pub threads: HashMap<String, ConversationThread>,
    pub recent_thread_summaries: HashMap<String, String>,
    pub pending_turn_controls: Vec<TurnControlRequest>,
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
                next_exchange_sequence: 1,
                next_record_sequence: 1,
                next_artifact_sequence: 1,
                next_branch_sequence: 1,
                next_turn_control_sequence: 1,
                root_started: false,
                root_last_record_id: None,
                branch_last_record_ids: HashMap::new(),
                recorder_warning_emitted: false,
                active_turn_id: None,
                active_thread: ConversationThreadRef::Mainline,
                threads,
                recent_thread_summaries: HashMap::new(),
                pending_turn_controls: Vec::new(),
            })),
        }
    }

    pub fn state(&self) -> Arc<Mutex<ConversationSessionState>> {
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

    pub fn mark_turn_active(&self, turn_id: TurnTraceId) {
        self.state
            .lock()
            .expect("conversation session lock")
            .active_turn_id = Some(turn_id);
    }

    pub fn active_turn_id(&self) -> Option<TurnTraceId> {
        self.state
            .lock()
            .expect("conversation session lock")
            .active_turn_id
            .clone()
    }

    pub fn clear_turn_if_active(&self, turn_id: &TurnTraceId) {
        let mut state = self.state.lock().expect("conversation session lock");
        if state.active_turn_id.as_ref() == Some(turn_id) {
            state.active_turn_id = None;
        }
    }

    pub fn request_turn_steer(
        &self,
        prompt: impl Into<String>,
    ) -> std::result::Result<TurnControlRequest, TurnControlRequestError> {
        let mut state = self.state.lock().expect("conversation session lock");
        let Some(turn_id) = state.active_turn_id.clone() else {
            return Err(TurnControlRequestError::NoActiveTurn);
        };
        let request = TurnControlRequest {
            control_id: TurnControlId::new(format!(
                "{}.control-{:04}",
                state.task_id.as_str(),
                state.next_turn_control_sequence
            ))
            .expect("generated turn control id"),
            turn_id,
            active_thread: state.active_thread.clone(),
            kind: TurnControlKind::Steer,
            prompt: Some(prompt.into()),
            captured_sequence: state.next_turn_control_sequence,
        };
        state.next_turn_control_sequence += 1;
        state.pending_turn_controls.push(request.clone());
        Ok(request)
    }

    pub fn request_turn_interrupt(
        &self,
    ) -> std::result::Result<TurnControlRequest, TurnControlRequestError> {
        let mut state = self.state.lock().expect("conversation session lock");
        let Some(turn_id) = state.active_turn_id.clone() else {
            return Err(TurnControlRequestError::NoActiveTurn);
        };
        let request = TurnControlRequest {
            control_id: TurnControlId::new(format!(
                "{}.control-{:04}",
                state.task_id.as_str(),
                state.next_turn_control_sequence
            ))
            .expect("generated turn control id"),
            turn_id,
            active_thread: state.active_thread.clone(),
            kind: TurnControlKind::Interrupt,
            prompt: None,
            captured_sequence: state.next_turn_control_sequence,
        };
        state.next_turn_control_sequence += 1;
        state.pending_turn_controls.push(request.clone());
        Ok(request)
    }

    pub fn take_turn_control_requests(&self, turn_id: &TurnTraceId) -> Vec<TurnControlRequest> {
        let mut state = self.state.lock().expect("conversation session lock");
        let mut retained = Vec::new();
        let mut matched = Vec::new();
        for request in state.pending_turn_controls.drain(..) {
            if &request.turn_id == turn_id {
                matched.push(request);
            } else {
                retained.push(request);
            }
        }
        state.pending_turn_controls = retained;
        matched
    }

    pub fn capture_candidate_from_turn_control(
        &self,
        request: &TurnControlRequest,
    ) -> Option<ThreadCandidate> {
        let prompt = request.prompt.as_ref()?.clone();
        let mut state = self.state.lock().expect("conversation session lock");
        let candidate_id = ThreadCandidateId::new(format!(
            "{}.candidate-{:04}",
            state.task_id.as_str(),
            state.next_candidate_sequence
        ))
        .expect("generated candidate id");
        let captured_sequence = state.next_candidate_sequence;
        state.next_candidate_sequence += 1;
        Some(ThreadCandidate {
            candidate_id,
            prompt,
            captured_from_turn_id: Some(request.turn_id.clone()),
            active_thread: request.active_thread.clone(),
            captured_sequence,
        })
    }

    pub fn next_exchange_id(&self, turn_id: &TurnTraceId) -> String {
        let mut state = self.state.lock().expect("conversation session lock");
        let exchange_id = format!(
            "{}.exchange-{:04}",
            turn_id.as_str(),
            state.next_exchange_sequence
        );
        state.next_exchange_sequence += 1;
        exchange_id
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

    pub fn next_artifact_id(&self, turn_id: &TurnTraceId) -> TraceArtifactId {
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
    use super::{
        ConversationSession, ConversationThread, ConversationThreadRef, ConversationThreadStatus,
        TaskTraceId, ThreadCandidateId, ThreadDecision, ThreadDecisionId, ThreadDecisionKind,
        ThreadMergeMode, TraceBranchId, TurnControlKind, TurnControlRequestError,
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

    #[test]
    fn turn_control_requests_attach_to_the_active_turn() {
        let session = ConversationSession::new(TaskTraceId::new("task-3").expect("task"));
        let turn_id = session.allocate_turn_id();
        session.mark_turn_active(turn_id.clone());

        let request = session
            .request_turn_steer("focus on the failing test")
            .expect("active turn should accept steering");

        assert_eq!(request.turn_id, turn_id);
        assert_eq!(request.kind, TurnControlKind::Steer);
        assert_eq!(
            session.take_turn_control_requests(&request.turn_id),
            vec![request]
        );
    }

    #[test]
    fn turn_control_requests_fail_closed_without_an_active_turn() {
        let session = ConversationSession::new(TaskTraceId::new("task-4").expect("task"));

        assert_eq!(
            session.request_turn_interrupt(),
            Err(TurnControlRequestError::NoActiveTurn)
        );
    }

    #[test]
    fn turn_control_candidates_remember_the_source_turn() {
        let session = ConversationSession::new(TaskTraceId::new("task-5").expect("task"));
        let turn_id = session.allocate_turn_id();
        session.mark_turn_active(turn_id.clone());
        let request = session
            .request_turn_steer("follow the branch")
            .expect("active turn should accept steering");

        let candidate = session
            .capture_candidate_from_turn_control(&request)
            .expect("steering prompt should become a candidate");

        assert_eq!(candidate.prompt, "follow the branch");
        assert_eq!(candidate.captured_from_turn_id, Some(turn_id));
        assert_eq!(candidate.active_thread, ConversationThreadRef::Mainline);
    }
}
