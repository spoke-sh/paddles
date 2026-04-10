use super::trace_recording::TraceSessionContextSlice;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecialistBrainRequest {
    pub user_prompt: String,
    pub workspace_root: PathBuf,
    pub active_profile_id: String,
    pub session_context: TraceSessionContextSlice,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecialistBrainCapability {
    Available,
    Unsupported { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecialistBrainNote {
    pub brain_id: String,
    pub note: String,
}

pub trait SpecialistBrain: Send + Sync {
    fn id(&self) -> &'static str;

    fn capability(&self, request: &SpecialistBrainRequest) -> SpecialistBrainCapability;

    fn runtime_note(&self, request: &SpecialistBrainRequest) -> Result<SpecialistBrainNote>;
}
