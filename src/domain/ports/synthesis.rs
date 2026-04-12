use crate::domain::model::{
    AppliedEdit, ExecutionGovernanceOutcome, InstructionFrame, TurnEventSink, TurnIntent,
};
use crate::domain::ports::context_gathering::EvidenceBundle;
use crate::domain::ports::planning::{GroundingRequirement, WorkspaceAction};
use anyhow::Result;
use std::sync::Arc;

/// Result from executing a workspace action through the synthesizer engine.
#[derive(Debug)]
pub struct WorkspaceActionResult {
    pub name: String,
    pub summary: String,
    pub applied_edit: Option<AppliedEdit>,
    pub governance_outcome: Option<ExecutionGovernanceOutcome>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SynthesisHandoff {
    pub recent_turns: Vec<String>,
    pub recent_thread_summary: Option<String>,
    pub instruction_frame: Option<InstructionFrame>,
    pub grounding: Option<GroundingRequirement>,
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
