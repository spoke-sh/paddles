use crate::domain::model::{
    ExecutionHandDescriptor, ExecutionHandDiagnostic, ExecutionHandKind, ExecutionHandOperation,
    ExecutionHandPhase, default_local_execution_hand_descriptors,
};
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Debug)]
pub struct ExecutionHandRegistry {
    diagnostics: Mutex<BTreeMap<ExecutionHandKind, ExecutionHandDiagnostic>>,
}

impl Default for ExecutionHandRegistry {
    fn default() -> Self {
        Self::new(default_local_execution_hand_descriptors())
    }
}

impl ExecutionHandRegistry {
    pub fn new(descriptors: impl IntoIterator<Item = ExecutionHandDescriptor>) -> Self {
        let diagnostics = descriptors
            .into_iter()
            .map(|descriptor| {
                (
                    descriptor.hand,
                    ExecutionHandDiagnostic::from_descriptor(&descriptor),
                )
            })
            .collect();
        Self {
            diagnostics: Mutex::new(diagnostics),
        }
    }

    pub fn diagnostics(&self) -> Vec<ExecutionHandDiagnostic> {
        self.diagnostics
            .lock()
            .expect("execution hand diagnostics lock")
            .values()
            .cloned()
            .collect()
    }

    pub fn diagnostic(&self, hand: ExecutionHandKind) -> Option<ExecutionHandDiagnostic> {
        self.diagnostics
            .lock()
            .expect("execution hand diagnostics lock")
            .get(&hand)
            .cloned()
    }

    pub fn record_phase(
        &self,
        hand: ExecutionHandKind,
        phase: ExecutionHandPhase,
        operation: ExecutionHandOperation,
        summary: impl Into<String>,
        last_error: Option<String>,
    ) {
        if let Some(diagnostic) = self
            .diagnostics
            .lock()
            .expect("execution hand diagnostics lock")
            .get_mut(&hand)
        {
            diagnostic.phase = phase;
            diagnostic.last_operation = Some(operation);
            diagnostic.summary = summary.into();
            diagnostic.last_error = last_error;
        }
    }

    pub fn record_ready(&self, hand: ExecutionHandKind, summary: impl Into<String>) {
        self.record_phase(
            hand,
            ExecutionHandPhase::Ready,
            ExecutionHandOperation::Provision,
            summary,
            None,
        );
    }

    pub fn record_executing(&self, hand: ExecutionHandKind, summary: impl Into<String>) {
        self.record_phase(
            hand,
            ExecutionHandPhase::Executing,
            ExecutionHandOperation::Execute,
            summary,
            None,
        );
    }

    pub fn record_recovering(&self, hand: ExecutionHandKind, summary: impl Into<String>) {
        self.record_phase(
            hand,
            ExecutionHandPhase::Recovering,
            ExecutionHandOperation::Recover,
            summary,
            None,
        );
    }

    pub fn record_degraded(&self, hand: ExecutionHandKind, error: impl Into<String>) {
        let error = error.into();
        self.record_phase(
            hand,
            ExecutionHandPhase::Degraded,
            ExecutionHandOperation::Degrade,
            error.clone(),
            Some(error),
        );
    }

    pub fn record_failed(&self, hand: ExecutionHandKind, error: impl Into<String>) {
        let error = error.into();
        self.record_phase(
            hand,
            ExecutionHandPhase::Failed,
            ExecutionHandOperation::Degrade,
            error.clone(),
            Some(error),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::ExecutionHandRegistry;
    use crate::domain::model::{
        ExecutionHandAuthority, ExecutionHandKind, ExecutionHandOperation, ExecutionHandPhase,
    };

    #[test]
    fn registry_bootstraps_consistent_diagnostics_for_local_execution_hands() {
        let registry = ExecutionHandRegistry::default();
        let diagnostics = registry.diagnostics();

        assert_eq!(diagnostics.len(), 3);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.hand == ExecutionHandKind::WorkspaceEditor
                && diagnostic.phase == ExecutionHandPhase::Described
                && diagnostic.authority == ExecutionHandAuthority::WorkspaceScoped
                && diagnostic.last_operation == Some(ExecutionHandOperation::Describe)
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.hand == ExecutionHandKind::TransportMediator
                && diagnostic.phase == ExecutionHandPhase::Described
                && diagnostic.authority == ExecutionHandAuthority::CredentialMediated
        }));
    }

    #[test]
    fn registry_records_lifecycle_transitions_without_redefining_state_names() {
        let registry = ExecutionHandRegistry::default();
        registry.record_ready(
            ExecutionHandKind::TerminalRunner,
            "terminal runner provisioned",
        );
        registry.record_executing(
            ExecutionHandKind::TerminalRunner,
            "terminal runner executing planner-owned shell command",
        );
        registry.record_recovering(
            ExecutionHandKind::TerminalRunner,
            "terminal runner recovering after shell spawn failure",
        );
        registry.record_degraded(
            ExecutionHandKind::TerminalRunner,
            "terminal runner degraded after repeated spawn failures",
        );

        let diagnostic = registry
            .diagnostic(ExecutionHandKind::TerminalRunner)
            .expect("terminal hand diagnostic");
        assert_eq!(diagnostic.phase, ExecutionHandPhase::Degraded);
        assert_eq!(
            diagnostic.last_operation,
            Some(ExecutionHandOperation::Degrade)
        );
        assert_eq!(
            diagnostic.last_error.as_deref(),
            Some("terminal runner degraded after repeated spawn failures")
        );
    }
}
