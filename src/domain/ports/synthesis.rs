use crate::domain::model::{CollaborationModeResult, InstructionFrame, TurnEventSink, TurnIntent};
use crate::domain::ports::context_gathering::EvidenceBundle;
use crate::domain::ports::planning::GroundingRequirement;
use anyhow::Result;
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SynthesisHandoff {
    pub recent_turns: Vec<String>,
    pub recent_thread_summary: Option<String>,
    pub collaboration: CollaborationModeResult,
    pub instruction_frame: Option<InstructionFrame>,
    pub grounding: Option<GroundingRequirement>,
}

/// Port for the synthesizer engine that authors final responses for a turn.
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
}
