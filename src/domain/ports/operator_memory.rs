use crate::domain::ports::planning::{InterpretationContext, OperatorMemoryDocument};
use std::path::Path;

/// Port for loading operator memory and building interpretation context.
pub trait OperatorMemory: Send + Sync {
    fn operator_memory_documents(&self, workspace_root: &Path) -> Vec<OperatorMemoryDocument>;

    fn build_interpretation_context(
        &self,
        user_prompt: &str,
        workspace_root: &Path,
    ) -> InterpretationContext;
}
