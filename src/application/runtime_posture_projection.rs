use crate::domain::model::{
    DelegatedWorkerProjection, EvalReport, EvalStatus, ExecutionGovernanceDecision,
    ExecutionHandDiagnostic, ExternalCapabilityResult,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePostureProjectionInput {
    pub capabilities: Vec<RuntimeCapabilityPostureEvent>,
    pub governance_decisions: Vec<ExecutionGovernanceDecision>,
    pub diagnostics: Vec<ExecutionHandDiagnostic>,
    pub external_results: Vec<ExternalCapabilityResult>,
    pub workers: Vec<DelegatedWorkerProjection>,
    pub eval_reports: Vec<EvalReport>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCapabilityPostureEvent {
    pub surface: String,
    pub capability_id: String,
    pub status: String,
    pub detail: String,
}

impl RuntimeCapabilityPostureEvent {
    pub fn available(
        surface: impl Into<String>,
        capability_id: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            surface: surface.into(),
            capability_id: capability_id.into(),
            status: "available".to_string(),
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeGovernancePostureEvent {
    pub subject: String,
    pub outcome: String,
    pub reason: String,
    pub detail: String,
}

impl From<&ExecutionGovernanceDecision> for RuntimeGovernancePostureEvent {
    fn from(decision: &ExecutionGovernanceDecision) -> Self {
        Self {
            subject: decision.subject(),
            outcome: decision.outcome.kind.label().to_string(),
            reason: decision.outcome.reason.clone(),
            detail: decision.detail(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeDiagnosticPostureEvent {
    pub hand: String,
    pub phase: String,
    pub authority: String,
    pub supported_operations: Vec<String>,
    pub last_operation: Option<String>,
    pub summary: String,
    pub last_error: Option<String>,
}

impl From<&ExecutionHandDiagnostic> for RuntimeDiagnosticPostureEvent {
    fn from(diagnostic: &ExecutionHandDiagnostic) -> Self {
        Self {
            hand: diagnostic.hand.label().to_string(),
            phase: diagnostic.phase.label().to_string(),
            authority: diagnostic.authority.label().to_string(),
            supported_operations: diagnostic
                .supported_operations
                .iter()
                .map(|operation| operation.label().to_string())
                .collect(),
            last_operation: diagnostic
                .last_operation
                .map(|operation| operation.label().to_string()),
            summary: diagnostic.summary.clone(),
            last_error: diagnostic.last_error.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeProvenancePostureEvent {
    pub capability_id: String,
    pub status: String,
    pub availability: String,
    pub summary: String,
    pub detail: String,
    pub sources: Vec<String>,
}

impl From<&ExternalCapabilityResult> for RuntimeProvenancePostureEvent {
    fn from(result: &ExternalCapabilityResult) -> Self {
        Self {
            capability_id: result.descriptor.id.clone(),
            status: result.status.label().to_string(),
            availability: result.descriptor.availability.label().to_string(),
            summary: result.summary.clone(),
            detail: result.detail.clone(),
            sources: result
                .sources
                .iter()
                .map(|source| source.locator.clone())
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeWorkerPostureEvent {
    pub worker_id: String,
    pub role_label: String,
    pub status: String,
    pub integration_status: Option<String>,
    pub artifact_count: usize,
    pub degraded: bool,
    pub progress_summary: String,
    pub latest_detail: String,
}

impl From<&DelegatedWorkerProjection> for RuntimeWorkerPostureEvent {
    fn from(worker: &DelegatedWorkerProjection) -> Self {
        Self {
            worker_id: worker.worker_id.clone(),
            role_label: worker.role_label.clone(),
            status: worker.status.label().to_string(),
            integration_status: worker
                .integration_status
                .map(|status| status.label().to_string()),
            artifact_count: worker.artifact_count,
            degraded: worker.degraded,
            progress_summary: worker.progress_summary.clone(),
            latest_detail: worker.latest_detail.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeEvalPostureEvent {
    pub scenario_id: String,
    pub status: String,
    pub outcomes: Vec<RuntimeEvalOutcomePostureEvent>,
}

impl From<&EvalReport> for RuntimeEvalPostureEvent {
    fn from(report: &EvalReport) -> Self {
        Self {
            scenario_id: report.scenario_id.clone(),
            status: eval_status_label(report.status).to_string(),
            outcomes: report
                .outcomes
                .iter()
                .map(RuntimeEvalOutcomePostureEvent::from)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeEvalOutcomePostureEvent {
    pub contract: String,
    pub status: String,
    pub message: String,
}

impl From<&crate::domain::model::EvalOutcome> for RuntimeEvalOutcomePostureEvent {
    fn from(outcome: &crate::domain::model::EvalOutcome) -> Self {
        Self {
            contract: outcome.contract.label().to_string(),
            status: eval_status_label(outcome.status).to_string(),
            message: outcome.message.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePostureProjectionSnapshot {
    pub capabilities: Vec<RuntimeCapabilityPostureEvent>,
    pub governance: Vec<RuntimeGovernancePostureEvent>,
    pub diagnostics: Vec<RuntimeDiagnosticPostureEvent>,
    pub provenance: Vec<RuntimeProvenancePostureEvent>,
    pub workers: Vec<RuntimeWorkerPostureEvent>,
    pub evals: Vec<RuntimeEvalPostureEvent>,
}

impl RuntimePostureProjectionSnapshot {
    pub fn operator_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.extend(self.capabilities.iter().map(|capability| {
            format!(
                "capability {}:{} {} - {}",
                capability.surface, capability.capability_id, capability.status, capability.detail
            )
        }));
        lines.extend(self.governance.iter().map(|governance| {
            format!(
                "governance {} {} - {}",
                governance.subject, governance.outcome, governance.reason
            )
        }));
        lines.extend(self.diagnostics.iter().map(|diagnostic| {
            format!(
                "diagnostic {} {} - {}",
                diagnostic.hand, diagnostic.phase, diagnostic.summary
            )
        }));
        lines.extend(self.provenance.iter().map(|provenance| {
            format!(
                "provenance {} {} - {}",
                provenance.capability_id, provenance.status, provenance.summary
            )
        }));
        lines.extend(self.workers.iter().map(|worker| {
            format!(
                "worker {} {} - {}",
                worker.worker_id, worker.status, worker.progress_summary
            )
        }));
        lines.extend(self.evals.iter().map(|eval| {
            format!(
                "eval {} {} - {} outcome(s)",
                eval.scenario_id,
                eval.status,
                eval.outcomes.len()
            )
        }));
        lines
    }
}

pub struct RuntimePostureProjectionService;

impl RuntimePostureProjectionService {
    pub fn project(input: RuntimePostureProjectionInput) -> RuntimePostureProjectionSnapshot {
        RuntimePostureProjectionSnapshot {
            capabilities: input.capabilities,
            governance: input
                .governance_decisions
                .iter()
                .map(RuntimeGovernancePostureEvent::from)
                .collect(),
            diagnostics: input
                .diagnostics
                .iter()
                .map(RuntimeDiagnosticPostureEvent::from)
                .collect(),
            provenance: input
                .external_results
                .iter()
                .map(RuntimeProvenancePostureEvent::from)
                .collect(),
            workers: input
                .workers
                .iter()
                .map(RuntimeWorkerPostureEvent::from)
                .collect(),
            evals: input
                .eval_reports
                .iter()
                .map(RuntimeEvalPostureEvent::from)
                .collect(),
        }
    }
}

fn eval_status_label(status: EvalStatus) -> &'static str {
    match status {
        EvalStatus::Passed => "passed",
        EvalStatus::Failed => "failed",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RuntimeCapabilityPostureEvent, RuntimePostureProjectionInput,
        RuntimePostureProjectionService,
    };
    use crate::domain::model::{
        DelegatedWorkerProjection, DelegatedWorkerStatus, EvalHarnessContract, EvalOutcome,
        EvalReport, EvalStatus, ExecutionGovernanceDecision, ExecutionGovernanceOutcome,
        ExecutionHandAuthority, ExecutionHandDiagnostic, ExecutionHandKind, ExecutionHandOperation,
        ExecutionHandPhase, ExecutionPermission, ExecutionPermissionRequest,
        ExecutionPermissionRequirement, ExternalCapabilityInvocation, ExternalCapabilityResult,
        ExternalCapabilitySourceRecord, WorkerIntegrationStatus,
        default_external_capability_descriptors,
    };
    use serde_json::json;

    #[test]
    fn runtime_posture_projection_exposes_runtime_facts_without_controller_authored_plans() {
        let input = RuntimePostureProjectionInput {
            capabilities: vec![RuntimeCapabilityPostureEvent::available(
                "external_capability",
                "web.search",
                "broker catalog advertised read-only web search",
            )],
            governance_decisions: vec![sample_governance_decision()],
            ..RuntimePostureProjectionInput::default()
        };

        let snapshot = RuntimePostureProjectionService::project(input);
        let operator_lines = snapshot.operator_lines();

        assert!(
            operator_lines
                .iter()
                .any(|line| line.contains("capability external_capability:web.search available"))
        );
        assert!(
            operator_lines
                .iter()
                .any(|line| line.contains("governance shell via terminal_runner allowed"))
        );
        assert!(
            operator_lines
                .iter()
                .all(|line| !line.contains("Updated Plan")
                    && !line.contains("Next step")
                    && !line.contains("I will"))
        );
    }

    #[test]
    fn runtime_posture_projection_snapshots_include_governance_diagnostics_provenance_worker_and_eval_outcomes()
     {
        let source = ExternalCapabilitySourceRecord {
            label: "Release notes".to_string(),
            locator: "https://example.test/releases".to_string(),
            snippet: "cached release notes".to_string(),
        };
        let input = RuntimePostureProjectionInput {
            governance_decisions: vec![sample_governance_decision()],
            diagnostics: vec![ExecutionHandDiagnostic {
                hand: ExecutionHandKind::WorkspaceEditor,
                phase: ExecutionHandPhase::Ready,
                authority: ExecutionHandAuthority::WorkspaceScoped,
                supported_operations: vec![ExecutionHandOperation::Execute],
                last_operation: Some(ExecutionHandOperation::Execute),
                summary: "workspace editor ready".to_string(),
                last_error: None,
            }],
            external_results: vec![sample_external_result(source.clone())],
            workers: vec![sample_worker_projection()],
            eval_reports: vec![EvalReport {
                scenario_id: "recursive-replay".to_string(),
                status: EvalStatus::Passed,
                outcomes: vec![EvalOutcome {
                    contract: EvalHarnessContract::Replay,
                    status: EvalStatus::Passed,
                    message: "replay reconstructed model-visible context".to_string(),
                }],
            }],
            ..RuntimePostureProjectionInput::default()
        };

        let snapshot = RuntimePostureProjectionService::project(input);

        assert_eq!(snapshot.governance[0].outcome, "allowed");
        assert_eq!(snapshot.diagnostics[0].phase, "ready");
        assert_eq!(snapshot.provenance[0].sources[0], source.locator);
        assert_eq!(
            snapshot.workers[0].integration_status.as_deref(),
            Some("integrated")
        );
        assert_eq!(snapshot.evals[0].outcomes[0].contract, "replay");
        assert_eq!(snapshot.evals[0].outcomes[0].status, "passed");
    }

    fn sample_governance_decision() -> ExecutionGovernanceDecision {
        let requirement = ExecutionPermissionRequirement::new(
            "run shell command",
            vec![ExecutionPermission::RunWorkspaceCommand],
        );
        ExecutionGovernanceDecision::new(
            Some("call-1".to_string()),
            Some("shell".to_string()),
            ExecutionPermissionRequest::new(ExecutionHandKind::TerminalRunner, requirement.clone()),
            ExecutionGovernanceOutcome::allowed(
                "bounded command allowed by local policy",
                requirement,
                vec![ExecutionPermission::RunWorkspaceCommand],
            ),
        )
    }

    fn sample_external_result(source: ExternalCapabilitySourceRecord) -> ExternalCapabilityResult {
        let descriptor = default_external_capability_descriptors()
            .into_iter()
            .find(|descriptor| descriptor.id == "web.search")
            .expect("web search descriptor");
        ExternalCapabilityResult::degraded(
            descriptor,
            ExternalCapabilityInvocation::new(
                "web.search",
                "confirm current release notes",
                json!({ "query": "paddles release notes" }),
            ),
            "cached web result used while capability is degraded",
            vec![source],
        )
    }

    fn sample_worker_projection() -> DelegatedWorkerProjection {
        DelegatedWorkerProjection {
            worker_id: "worker-1".to_string(),
            role_label: "Worker".to_string(),
            ownership_summary: "Own src/application/runtime_posture_projection.rs".to_string(),
            read_scopes: vec!["src/application".to_string()],
            write_scopes: vec!["src/application/runtime_posture_projection.rs".to_string()],
            parent_thread: "mainline".to_string(),
            worker_thread: "worker-thread".to_string(),
            status: DelegatedWorkerStatus::Integrated,
            progress_summary: "Worker evidence integrated.".to_string(),
            latest_detail: "Applied projection snapshot changes.".to_string(),
            artifact_count: 3,
            completion_recorded: true,
            integration_status: Some(WorkerIntegrationStatus::Integrated),
            degraded: false,
        }
    }
}
