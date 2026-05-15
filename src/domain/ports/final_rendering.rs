use crate::domain::model::{InstructionFrame, TurnContract, TurnEventSink, TurnIntent};
use crate::domain::ports::action_selection::GroundingRequirement;
use crate::domain::ports::retrieval::EvidenceBundle;
use anyhow::Result;
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FinalRenderingHandoff {
    pub recent_turns: Vec<String>,
    pub recent_thread_summary: Option<String>,
    pub turn_contract: TurnContract,
    pub instruction_frame: Option<InstructionFrame>,
    pub grounding: Option<GroundingRequirement>,
}

/// Port for the final-rendering engine that authors final responses for a turn.
pub trait FinalRenderingEngine: Send + Sync {
    fn set_verbose(&self, level: u8);

    fn respond_for_turn(
        &self,
        prompt: &str,
        turn_intent: TurnIntent,
        gathered_evidence: Option<&EvidenceBundle>,
        handoff: &FinalRenderingHandoff,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String>;

    fn recent_turn_summaries(&self) -> Result<Vec<String>>;
}
