use crate::domain::model::{ExecutionHandDescriptor, ExecutionHandDiagnostic};

/// Shared descriptive boundary for local execution hands.
///
/// This trait intentionally exposes the stable descriptive and diagnostic
/// surface first. Later stories can migrate concrete workspace, terminal, and
/// transport adapters onto the contract without redefining the lifecycle
/// vocabulary.
pub trait ExecutionHand: Send + Sync {
    fn describe(&self) -> ExecutionHandDescriptor;
    fn diagnostic(&self) -> ExecutionHandDiagnostic;
}
