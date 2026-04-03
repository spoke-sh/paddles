use crate::domain::model::{TurnEventSink, TurnIntent};
use crate::domain::ports::context_gathering::EvidenceBundle;
use crate::domain::ports::planning::WorkspaceAction;
use anyhow::Result;
use std::sync::Arc;

/// Result from executing a workspace action through the synthesizer engine.
#[derive(Debug)]
pub struct WorkspaceActionResult {
    pub name: String,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SynthesisHandoff {
    pub recent_turns: Vec<String>,
    pub recent_thread_summary: Option<String>,
}

/// Port for the synthesizer engine that drives conversation turns and workspace actions.
pub trait SynthesizerEngine: Send + Sync {
    fn set_verbose(&self, level: u8);

    fn respond_for_turn(
        &self,
        prompt: &str,
        turn_intent: TurnIntent,
        gathered_evidence: Option<&EvidenceBundle>,
        handoff: &SynthesisHandoff,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String>;

    fn recent_turn_summaries(&self) -> Result<Vec<String>>;

    fn execute_workspace_action(&self, action: &WorkspaceAction) -> Result<WorkspaceActionResult>;
}
