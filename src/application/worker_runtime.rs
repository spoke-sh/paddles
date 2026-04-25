use super::execution_policy::ExecutionPolicyEvaluator;
use crate::domain::model::{
    ConversationThreadRef, DelegationEvidencePolicy, DelegationGovernancePolicy,
    ExecutionGovernanceSnapshot, ExecutionPolicy, ExecutionPolicyDecisionKind,
    ExecutionPolicyEvaluationInput, ExternalCapabilityDescriptor, TraceBranchId,
    TraceWorkerArtifact, TraceWorkerIntegration, TraceWorkerLifecycle, WorkerDelegationContract,
    WorkerDelegationRequest, WorkerIntegrationStatus, WorkerLifecycleOperation,
    WorkerLifecycleResult, WorkerLifecycleResultStatus, WorkerOwnership,
    default_local_execution_policy,
};
use crate::domain::ports::{EvidenceBundle, EvidenceItem, WorkspaceCapabilitySurface};
use anyhow::{Result, bail};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkerRuntimeBudget {
    pub max_steps: usize,
    pub max_evidence_items: usize,
}

impl WorkerRuntimeBudget {
    pub fn new(max_steps: usize, max_evidence_items: usize) -> Self {
        Self {
            max_steps,
            max_evidence_items,
        }
    }
}

impl Default for WorkerRuntimeBudget {
    fn default() -> Self {
        Self {
            max_steps: 6,
            max_evidence_items: 6,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkerRuntimeCapabilityPosture {
    pub workspace: WorkspaceCapabilitySurface,
    pub external_capabilities: Vec<ExternalCapabilityDescriptor>,
}

impl WorkerRuntimeCapabilityPosture {
    pub fn new(
        workspace: WorkspaceCapabilitySurface,
        external_capabilities: Vec<ExternalCapabilityDescriptor>,
    ) -> Self {
        let mut external_capabilities = external_capabilities;
        external_capabilities.sort_by(|left, right| left.id.cmp(&right.id));
        external_capabilities.dedup_by(|left, right| left.id == right.id);
        Self {
            workspace,
            external_capabilities,
        }
    }

    fn has_workspace_action(&self, action: &str) -> bool {
        self.workspace
            .actions
            .iter()
            .any(|candidate| candidate.action == action)
    }

    fn has_workspace_tool(&self, tool: &str) -> bool {
        self.workspace.has_tool(tool)
    }

    fn external_capability(&self, capability_id: &str) -> Option<&ExternalCapabilityDescriptor> {
        self.external_capabilities
            .iter()
            .find(|descriptor| descriptor.id == capability_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerRuntimeContext {
    pub governance: DelegationGovernancePolicy,
    pub execution_policy: ExecutionPolicy,
    pub capability_posture: WorkerRuntimeCapabilityPosture,
    pub budget: WorkerRuntimeBudget,
}

impl WorkerRuntimeContext {
    pub fn new(
        governance: DelegationGovernancePolicy,
        execution_policy: ExecutionPolicy,
        capability_posture: WorkerRuntimeCapabilityPosture,
        budget: WorkerRuntimeBudget,
    ) -> Self {
        Self {
            governance,
            execution_policy,
            capability_posture,
            budget,
        }
    }

    pub fn inherit_from_parent(
        snapshot: &ExecutionGovernanceSnapshot,
        execution_policy: ExecutionPolicy,
        capability_posture: WorkerRuntimeCapabilityPosture,
        evidence_policy: DelegationEvidencePolicy,
        budget: WorkerRuntimeBudget,
    ) -> Self {
        Self::new(
            DelegationGovernancePolicy::inherit_from_parent(snapshot, evidence_policy),
            execution_policy,
            capability_posture,
            budget,
        )
    }

    pub fn from_contract(contract: &WorkerDelegationContract, budget: WorkerRuntimeBudget) -> Self {
        Self::new(
            contract.governance.clone(),
            default_local_execution_policy(),
            WorkerRuntimeCapabilityPosture::default(),
            budget,
        )
    }

    pub fn authorize_workspace_action(&self, action: &str) -> WorkerRuntimeAuthorityDecision {
        if self.capability_posture.has_workspace_action(action) {
            WorkerRuntimeAuthorityDecision::allow(format!(
                "workspace action `{action}` is present in the inherited parent capability surface"
            ))
        } else {
            WorkerRuntimeAuthorityDecision::deny(format!(
                "workspace action `{action}` was not available to the parent turn"
            ))
        }
    }

    pub fn authorize_workspace_tool(&self, tool: &str) -> WorkerRuntimeAuthorityDecision {
        if self.capability_posture.has_workspace_tool(tool) {
            WorkerRuntimeAuthorityDecision::allow(format!(
                "workspace tool `{tool}` is present in the inherited parent capability surface"
            ))
        } else {
            WorkerRuntimeAuthorityDecision::deny(format!(
                "workspace tool `{tool}` was not available to the parent turn"
            ))
        }
    }

    pub fn authorize_external_capability(
        &self,
        capability_id: &str,
    ) -> WorkerRuntimeAuthorityDecision {
        match self.capability_posture.external_capability(capability_id) {
            Some(descriptor) if descriptor.availability.is_usable() => {
                WorkerRuntimeAuthorityDecision::allow(format!(
                    "external capability `{capability_id}` is available in the inherited parent capability surface"
                ))
            }
            Some(descriptor) => WorkerRuntimeAuthorityDecision::deny(format!(
                "external capability `{capability_id}` is {} for the parent turn",
                descriptor.availability.label()
            )),
            None => WorkerRuntimeAuthorityDecision::deny(format!(
                "external capability `{capability_id}` was not available to the parent turn"
            )),
        }
    }

    pub fn authorize_execution_policy(
        &self,
        input: &ExecutionPolicyEvaluationInput,
    ) -> WorkerRuntimeAuthorityDecision {
        let decision = ExecutionPolicyEvaluator::evaluate(&self.execution_policy, input);
        if decision.kind == ExecutionPolicyDecisionKind::Deny {
            WorkerRuntimeAuthorityDecision::deny(format!(
                "execution policy denied worker execution: {}",
                decision.reason
            ))
        } else {
            WorkerRuntimeAuthorityDecision::allow(format!(
                "execution policy permitted worker execution as {}: {}",
                decision.kind.label(),
                decision.reason
            ))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerRuntimeAuthorityDecision {
    pub allowed: bool,
    pub reason: String,
}

impl WorkerRuntimeAuthorityDecision {
    pub fn allow(reason: impl Into<String>) -> Self {
        Self {
            allowed: true,
            reason: reason.into(),
        }
    }

    pub fn deny(reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            reason: reason.into(),
        }
    }

    pub fn allowed(&self) -> bool {
        self.allowed
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerEvidenceIntegrationStatus {
    Accepted,
    Rejected,
    NeedsIntegration,
}

impl WorkerEvidenceIntegrationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::NeedsIntegration => "needs-integration",
        }
    }

    fn trace_status(self) -> WorkerIntegrationStatus {
        match self {
            Self::Accepted => WorkerIntegrationStatus::Integrated,
            Self::Rejected => WorkerIntegrationStatus::Rejected,
            Self::NeedsIntegration => WorkerIntegrationStatus::NeedsIntegration,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerEvidenceIntegrationRequest {
    pub worker_id: String,
    pub parent_thread: ConversationThreadRef,
    pub worker_thread: ConversationThreadRef,
    pub artifacts: Vec<TraceWorkerArtifact>,
    pub requested_status: WorkerEvidenceIntegrationStatus,
    pub detail: String,
    pub worker_ownership: Option<WorkerOwnership>,
    pub active_parent_ownerships: Vec<WorkerOwnership>,
}

impl WorkerEvidenceIntegrationRequest {
    pub fn new(
        worker_id: impl Into<String>,
        parent_thread: ConversationThreadRef,
        worker_thread: ConversationThreadRef,
        artifacts: Vec<TraceWorkerArtifact>,
    ) -> Self {
        Self {
            worker_id: worker_id.into(),
            parent_thread,
            worker_thread,
            artifacts,
            requested_status: WorkerEvidenceIntegrationStatus::NeedsIntegration,
            detail: "Worker output is awaiting parent integration.".to_string(),
            worker_ownership: None,
            active_parent_ownerships: Vec::new(),
        }
    }

    pub fn with_requested_status(mut self, status: WorkerEvidenceIntegrationStatus) -> Self {
        self.requested_status = status;
        self
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }

    pub fn with_worker_ownership(mut self, ownership: WorkerOwnership) -> Self {
        self.worker_ownership = Some(ownership);
        self
    }

    pub fn with_active_parent_ownerships(mut self, ownerships: Vec<WorkerOwnership>) -> Self {
        self.active_parent_ownerships = ownerships;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerEvidenceIntegrationOutcome {
    pub status: WorkerEvidenceIntegrationStatus,
    pub evidence: EvidenceBundle,
    pub integration: TraceWorkerIntegration,
    pub applied_edit_count: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WorkerEvidenceIntegrator;

impl WorkerEvidenceIntegrator {
    pub fn integrate(
        request: WorkerEvidenceIntegrationRequest,
    ) -> Result<WorkerEvidenceIntegrationOutcome> {
        if request.worker_id.trim().is_empty() {
            bail!("worker evidence integration requires a worker id");
        }
        if request.artifacts.is_empty() {
            bail!("worker evidence integration requires at least one worker artifact");
        }

        let mut status = request.requested_status;
        let mut warnings = Vec::new();
        let mut detail = if request.detail.trim().is_empty() {
            format!("Parent recorded worker output as {}.", status.label())
        } else {
            request.detail.clone()
        };

        let conflicts = integration_conflicts(
            request.worker_ownership.as_ref(),
            request.active_parent_ownerships.as_slice(),
        );
        if !conflicts.is_empty() {
            status = WorkerEvidenceIntegrationStatus::Rejected;
            let conflict_summary = format!("ownership conflict on [{}]", conflicts.join(", "));
            warnings.push(format!(
                "Worker `{}` integration rejected due to {conflict_summary}.",
                request.worker_id
            ));
            detail = format!("{conflict_summary}; {detail}");
        }

        let visible_artifacts = request
            .artifacts
            .iter()
            .filter(|artifact| artifact.record.parent_visible)
            .collect::<Vec<_>>();
        if visible_artifacts.is_empty() {
            bail!("worker evidence integration requires parent-visible artifacts");
        }
        let evidence_items = visible_artifacts
            .iter()
            .enumerate()
            .map(|(index, artifact)| EvidenceItem {
                source: format!(
                    "worker:{}:{}:{}",
                    request.worker_id,
                    artifact.record.kind.label(),
                    artifact.record.label
                ),
                snippet: worker_artifact_snippet(&artifact.artifact),
                rationale: format!(
                    "worker {} {} evidence: {}",
                    request.worker_id,
                    status.label(),
                    artifact.record.summary
                ),
                rank: index + 1,
            })
            .collect::<Vec<_>>();
        let integrated_artifact_ids = if status == WorkerEvidenceIntegrationStatus::Accepted {
            visible_artifacts
                .iter()
                .map(|artifact| artifact.artifact.artifact_id.clone())
                .collect()
        } else {
            Vec::new()
        };
        let evidence = EvidenceBundle::new(
            format!(
                "Worker {} {} with {} parent-visible artifact(s).",
                request.worker_id,
                status.label(),
                evidence_items.len()
            ),
            evidence_items,
        )
        .with_warnings(warnings);

        Ok(WorkerEvidenceIntegrationOutcome {
            status,
            evidence,
            integration: TraceWorkerIntegration {
                worker_id: request.worker_id,
                parent_thread: request.parent_thread,
                worker_thread: request.worker_thread,
                status: status.trace_status(),
                detail,
                integrated_artifact_ids,
            },
            applied_edit_count: 0,
        })
    }
}

fn integration_conflicts(
    worker_ownership: Option<&WorkerOwnership>,
    active_parent_ownerships: &[WorkerOwnership],
) -> Vec<String> {
    let Some(worker_ownership) = worker_ownership else {
        return Vec::new();
    };
    let mut conflicts = active_parent_ownerships
        .iter()
        .flat_map(|parent_ownership| worker_ownership.conflicting_write_scopes(parent_ownership))
        .collect::<Vec<_>>();
    conflicts.sort();
    conflicts.dedup();
    conflicts
}

fn worker_artifact_snippet(artifact: &crate::domain::model::ArtifactEnvelope) -> String {
    artifact
        .inline_content
        .clone()
        .unwrap_or_else(|| artifact.summary.clone())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerRuntimeSpawnRequest {
    pub instruction: String,
    pub contract: WorkerDelegationContract,
    pub parent_thread: ConversationThreadRef,
    pub budget: WorkerRuntimeBudget,
    pub context: WorkerRuntimeContext,
}

impl WorkerRuntimeSpawnRequest {
    pub fn new(instruction: impl Into<String>, contract: WorkerDelegationContract) -> Self {
        let budget = WorkerRuntimeBudget::default();
        let context = WorkerRuntimeContext::from_contract(&contract, budget);
        Self {
            instruction: instruction.into(),
            contract,
            parent_thread: ConversationThreadRef::Mainline,
            budget,
            context,
        }
    }

    pub fn with_parent_thread(mut self, parent_thread: ConversationThreadRef) -> Self {
        self.parent_thread = parent_thread;
        self
    }

    pub fn with_budget(mut self, budget: WorkerRuntimeBudget) -> Self {
        self.budget = budget;
        self.context.budget = budget;
        self
    }

    pub fn with_inherited_context(mut self, context: WorkerRuntimeContext) -> Self {
        self.contract.governance = context.governance.clone();
        self.budget = context.budget;
        self.context = context;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerRuntimeSpawnOutcome {
    pub worker_id: String,
    pub worker_thread: ConversationThreadRef,
    pub lifecycle: TraceWorkerLifecycle,
    pub budget: WorkerRuntimeBudget,
    pub context: WorkerRuntimeContext,
}

pub trait WorkerRuntimePort: Send + Sync {
    fn spawn(&self, request: WorkerRuntimeSpawnRequest) -> Result<WorkerRuntimeSpawnOutcome>;
}

#[derive(Debug, Default)]
pub struct BoundedWorkerRuntime {
    next_worker: AtomicUsize,
}

impl WorkerRuntimePort for BoundedWorkerRuntime {
    fn spawn(&self, request: WorkerRuntimeSpawnRequest) -> Result<WorkerRuntimeSpawnOutcome> {
        if request.instruction.trim().is_empty() {
            bail!("worker spawn instruction cannot be empty");
        }
        if request.budget.max_steps == 0 || request.budget.max_evidence_items == 0 {
            bail!("worker runtime budget must allow at least one step and evidence item");
        }
        if request.contract.governance != request.context.governance {
            bail!("worker delegation contract governance must match inherited worker context");
        }
        if request.budget != request.context.budget {
            bail!("worker runtime budget must match inherited worker context budget");
        }

        let sequence = self.next_worker.fetch_add(1, Ordering::SeqCst) + 1;
        let worker_id = format!("worker-{sequence}");
        let worker_thread = ConversationThreadRef::Branch(
            TraceBranchId::new(format!("{worker_id}-thread")).expect("generated worker branch id"),
        );
        let lifecycle = TraceWorkerLifecycle {
            request: WorkerDelegationRequest::spawn(request.instruction, request.contract),
            result: WorkerLifecycleResult::new(
                WorkerLifecycleOperation::Spawn,
                WorkerLifecycleResultStatus::Accepted,
                Some(worker_id.clone()),
                format!("Spawned {worker_id} on a bounded recursive worker thread."),
            ),
            parent_thread: request.parent_thread,
            worker_thread: worker_thread.clone(),
        };

        Ok(WorkerRuntimeSpawnOutcome {
            worker_id,
            worker_thread,
            lifecycle,
            budget: request.budget,
            context: request.context,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BoundedWorkerRuntime, WorkerRuntimeBudget, WorkerRuntimePort, WorkerRuntimeSpawnRequest,
    };
    use crate::domain::model::{
        ArtifactEnvelope, ArtifactKind, DelegationEvidencePolicy, DelegationGovernancePolicy,
        DelegationIntegrationOwner, ExecutionApprovalPolicy, ExecutionGovernanceProfile,
        ExecutionGovernanceSnapshot, ExecutionPermissionReuseScope, ExecutionPolicy,
        ExecutionPolicyDecisionKind, ExecutionPolicyEvaluationInput, ExecutionPolicyMatcher,
        ExecutionPolicyRule, ExecutionSandboxMode, ExternalCapabilityCatalog,
        ExternalCapabilityCatalogConfig, TraceArtifactId, TraceBranchId, TraceRecordKind,
        TraceWorkerArtifact, WorkerArtifactKind, WorkerArtifactRecord, WorkerDelegationContract,
        WorkerDelegationRequest, WorkerIntegrationStatus, WorkerLifecycleOperation,
        WorkerLifecycleResultStatus, WorkerOwnership, WorkerRole,
    };
    use crate::domain::ports::{
        WorkspaceActionCapability, WorkspaceCapabilitySurface, WorkspaceToolCapability,
    };
    use paddles_conversation::ConversationThreadRef;

    #[test]
    fn worker_runtime_lifecycle_creates_bounded_worker_request_through_port() {
        let runtime = BoundedWorkerRuntime::default();
        let request = WorkerRuntimeSpawnRequest::new(
            "Audit parser lineage",
            worker_contract("Own parser lineage"),
        )
        .with_budget(WorkerRuntimeBudget::new(3, 4))
        .with_parent_thread(ConversationThreadRef::Mainline);

        let outcome = runtime.spawn(request).expect("spawn worker");

        assert_eq!(outcome.worker_id, "worker-1");
        assert_eq!(outcome.budget.max_steps, 3);
        assert_eq!(outcome.budget.max_evidence_items, 4);
        assert_eq!(
            outcome.lifecycle.request.operation,
            WorkerLifecycleOperation::Spawn
        );
        assert!(matches!(
            outcome.lifecycle.request,
            WorkerDelegationRequest {
                contract: Some(_),
                ..
            }
        ));
        assert_eq!(
            outcome.lifecycle.result.status,
            WorkerLifecycleResultStatus::Accepted
        );
        assert_eq!(
            outcome.lifecycle.result.worker_id.as_deref(),
            Some("worker-1")
        );
    }

    #[test]
    fn worker_trace_lifecycle_uses_existing_delegation_vocabulary() {
        let runtime = BoundedWorkerRuntime::default();
        let outcome = runtime
            .spawn(WorkerRuntimeSpawnRequest::new(
                "Inspect workspace boundaries",
                worker_contract("Own boundary inspection"),
            ))
            .expect("spawn worker");

        let record = TraceRecordKind::WorkerLifecycleRecorded(outcome.lifecycle.clone());
        let encoded = serde_json::to_string(&record).expect("serialize lifecycle");

        assert!(encoded.contains("WorkerLifecycleRecorded"));
        assert!(encoded.contains("\"operation\":\"spawn\""));
        assert!(encoded.contains("\"status\":\"accepted\""));
        assert_eq!(
            outcome.lifecycle.result.operation,
            WorkerLifecycleOperation::Spawn
        );
    }

    #[test]
    fn worker_inherits_governance_execution_policy_capabilities_and_budget() {
        let runtime = BoundedWorkerRuntime::default();
        let budget = WorkerRuntimeBudget::new(2, 3);
        let execution_policy = worker_execution_policy();
        let capability_posture = worker_capability_posture();
        let inherited_context = super::WorkerRuntimeContext::inherit_from_parent(
            &sample_governance_snapshot(),
            execution_policy.clone(),
            capability_posture.clone(),
            worker_evidence_policy(),
            budget,
        );
        let request = WorkerRuntimeSpawnRequest::new(
            "Inspect parser boundary without widening authority",
            worker_contract_with_governance(
                "Own parser boundary",
                inherited_context.governance.clone(),
            ),
        )
        .with_inherited_context(inherited_context.clone())
        .with_parent_thread(ConversationThreadRef::Mainline);

        let outcome = runtime.spawn(request).expect("spawn worker");

        assert_eq!(outcome.budget, budget);
        assert_eq!(outcome.context, inherited_context);
        assert_eq!(outcome.context.execution_policy, execution_policy);
        assert_eq!(outcome.context.capability_posture, capability_posture);
        assert_eq!(
            outcome
                .lifecycle
                .request
                .contract
                .as_ref()
                .map(|contract| &contract.governance),
            Some(&inherited_context.governance)
        );
    }

    #[test]
    fn worker_authority_bounds_reject_capabilities_absent_or_unavailable_to_parent() {
        let runtime = BoundedWorkerRuntime::default();
        let inherited_context = super::WorkerRuntimeContext::inherit_from_parent(
            &sample_governance_snapshot(),
            worker_execution_policy(),
            worker_capability_posture(),
            worker_evidence_policy(),
            WorkerRuntimeBudget::new(4, 4),
        );
        let outcome = runtime
            .spawn(
                WorkerRuntimeSpawnRequest::new(
                    "Probe only parent-visible capabilities",
                    worker_contract_with_governance(
                        "Own capability probe",
                        inherited_context.governance.clone(),
                    ),
                )
                .with_inherited_context(inherited_context),
            )
            .expect("spawn worker");

        assert!(
            outcome
                .context
                .authorize_workspace_action("inspect")
                .allowed()
        );
        assert!(
            !outcome
                .context
                .authorize_workspace_action("write_file")
                .allowed()
        );
        assert!(outcome.context.authorize_workspace_tool("rg").allowed());
        assert!(!outcome.context.authorize_workspace_tool("git").allowed());
        assert!(
            outcome
                .context
                .authorize_external_capability("web.search")
                .allowed()
        );
        assert!(
            !outcome
                .context
                .authorize_external_capability("mcp.tool")
                .allowed()
        );
        assert!(
            !outcome
                .context
                .authorize_external_capability("missing.fabric")
                .allowed()
        );
        assert!(
            outcome
                .context
                .authorize_execution_policy(&ExecutionPolicyEvaluationInput::command_for_tool(
                    "shell",
                    ["cargo", "test"],
                ))
                .allowed()
        );
        assert!(
            !outcome
                .context
                .authorize_execution_policy(&ExecutionPolicyEvaluationInput::command_for_tool(
                    "shell",
                    ["rm", "-rf", "/"],
                ))
                .allowed()
        );
    }

    #[test]
    fn worker_evidence_integration_projects_outputs_into_parent_loop_evidence_with_statuses() {
        let parent_thread = ConversationThreadRef::Mainline;
        let worker_thread =
            ConversationThreadRef::Branch(TraceBranchId::new("worker-1-thread").expect("branch"));
        let artifacts = worker_artifacts("worker-1");

        for (status, trace_status, expected_integrated_count) in [
            (
                super::WorkerEvidenceIntegrationStatus::Accepted,
                WorkerIntegrationStatus::Integrated,
                artifacts.len(),
            ),
            (
                super::WorkerEvidenceIntegrationStatus::Rejected,
                WorkerIntegrationStatus::Rejected,
                0,
            ),
            (
                super::WorkerEvidenceIntegrationStatus::NeedsIntegration,
                WorkerIntegrationStatus::NeedsIntegration,
                0,
            ),
        ] {
            let outcome = super::WorkerEvidenceIntegrator::integrate(
                super::WorkerEvidenceIntegrationRequest::new(
                    "worker-1",
                    parent_thread.clone(),
                    worker_thread.clone(),
                    artifacts.clone(),
                )
                .with_requested_status(status)
                .with_detail(format!(
                    "Parent recorded worker output as {}",
                    status.label()
                )),
            )
            .expect("integrate worker evidence");

            assert_eq!(outcome.status, status);
            assert_eq!(outcome.integration.status, trace_status);
            assert_eq!(outcome.evidence.items.len(), artifacts.len());
            assert_eq!(
                outcome.integration.integrated_artifact_ids.len(),
                expected_integrated_count
            );
            assert!(outcome.evidence.items.iter().all(|item| {
                item.source.starts_with("worker:worker-1")
                    && item.rationale.contains(status.label())
            }));
        }
    }

    #[test]
    fn worker_integration_conflicts_reject_parent_owned_write_scope_without_applying_worker_artifacts()
     {
        let parent_thread = ConversationThreadRef::Mainline;
        let worker_thread = ConversationThreadRef::Branch(
            TraceBranchId::new("worker-conflict-thread").expect("branch"),
        );
        let worker_ownership = WorkerOwnership::new(
            "Worker proposes runtime edits",
            vec!["src/application".to_string()],
            vec!["src/application/worker_runtime.rs".to_string()],
            DelegationIntegrationOwner::Parent,
        );
        let parent_ownership = WorkerOwnership::new(
            "Parent owns runtime edits",
            vec!["src/application".to_string()],
            vec!["src/application".to_string()],
            DelegationIntegrationOwner::Parent,
        );

        let outcome = super::WorkerEvidenceIntegrator::integrate(
            super::WorkerEvidenceIntegrationRequest::new(
                "worker-conflict",
                parent_thread,
                worker_thread,
                worker_artifacts("worker-conflict"),
            )
            .with_requested_status(super::WorkerEvidenceIntegrationStatus::Accepted)
            .with_detail("Parent attempted to accept a conflicting worker proposal.")
            .with_worker_ownership(worker_ownership)
            .with_active_parent_ownerships(vec![parent_ownership]),
        )
        .expect("integrate worker evidence");

        assert_eq!(
            outcome.status,
            super::WorkerEvidenceIntegrationStatus::Rejected
        );
        assert_eq!(
            outcome.integration.status,
            WorkerIntegrationStatus::Rejected
        );
        assert!(outcome.integration.integrated_artifact_ids.is_empty());
        assert_eq!(outcome.applied_edit_count, 0);
        assert!(
            outcome
                .evidence
                .warnings
                .iter()
                .any(|warning| warning.contains("ownership conflict"))
        );
        assert!(
            outcome
                .evidence
                .items
                .iter()
                .all(|item| item.rationale.contains("rejected"))
        );
    }

    fn worker_contract(summary: &str) -> WorkerDelegationContract {
        worker_contract_with_governance(
            summary,
            DelegationGovernancePolicy::inherit_from_parent(
                &sample_governance_snapshot(),
                worker_evidence_policy(),
            ),
        )
    }

    fn worker_contract_with_governance(
        summary: &str,
        governance: DelegationGovernancePolicy,
    ) -> WorkerDelegationContract {
        WorkerDelegationContract::new(
            WorkerRole::new("worker", "Worker", "Run bounded delegated work."),
            WorkerOwnership::new(
                summary,
                vec!["src".to_string()],
                vec!["src/application".to_string()],
                DelegationIntegrationOwner::Parent,
            ),
            governance,
        )
    }

    fn sample_governance_snapshot() -> ExecutionGovernanceSnapshot {
        ExecutionGovernanceSnapshot::new(
            "test-profile",
            "test-profile",
            ExecutionGovernanceProfile::new(
                ExecutionSandboxMode::WorkspaceWrite,
                ExecutionApprovalPolicy::OnRequest,
                vec![ExecutionPermissionReuseScope::Turn],
                None,
            ),
        )
    }

    fn worker_evidence_policy() -> DelegationEvidencePolicy {
        DelegationEvidencePolicy::new(
            "Worker evidence remains parent-visible.",
            vec![
                WorkerArtifactKind::ToolCall,
                WorkerArtifactKind::ToolOutput,
                WorkerArtifactKind::CompletionSummary,
            ],
        )
    }

    fn worker_execution_policy() -> ExecutionPolicy {
        ExecutionPolicy::new(vec![
            ExecutionPolicyRule::new(
                "allow-cargo-test",
                ExecutionPolicyMatcher::command_prefix(["cargo", "test"]),
                ExecutionPolicyDecisionKind::Allow,
                "tests are allowed parent verification",
            ),
            ExecutionPolicyRule::new(
                "deny-root-removal",
                ExecutionPolicyMatcher::command_prefix(["rm", "-rf", "/"]),
                ExecutionPolicyDecisionKind::Deny,
                "root removal is outside delegated authority",
            ),
        ])
    }

    fn worker_capability_posture() -> super::WorkerRuntimeCapabilityPosture {
        super::WorkerRuntimeCapabilityPosture::new(
            WorkspaceCapabilitySurface {
                actions: vec![WorkspaceActionCapability::new(
                    "inspect",
                    "Read-only inspection",
                    false,
                )],
                tools: vec![WorkspaceToolCapability::new(
                    "rg",
                    "Search workspace text",
                    None,
                )],
                notes: vec!["parent turn exposed only inspection and rg".to_string()],
            },
            ExternalCapabilityCatalog::from_local_configuration(
                &ExternalCapabilityCatalogConfig::default().enable("web.search"),
            )
            .descriptors(),
        )
    }

    fn worker_artifacts(worker_id: &str) -> Vec<TraceWorkerArtifact> {
        vec![
            TraceWorkerArtifact {
                record: WorkerArtifactRecord::tool_output(
                    worker_id,
                    "rg",
                    "Located the recursive worker evidence boundary.",
                ),
                artifact: worker_artifact(
                    worker_id,
                    1,
                    ArtifactKind::ToolOutput,
                    "worker tool output",
                    "src/application/worker_runtime.rs contains the worker runtime.",
                ),
            },
            TraceWorkerArtifact {
                record: WorkerArtifactRecord::completion_summary(
                    worker_id,
                    "Worker recommends parent integration of the runtime evidence.",
                    vec![
                        "Parent should review and integrate; no worker edits applied.".to_string(),
                    ],
                ),
                artifact: worker_artifact(
                    worker_id,
                    2,
                    ArtifactKind::EvidenceBundle,
                    "worker completion",
                    "Finding: worker outputs are evidence for the parent loop.",
                ),
            },
        ]
    }

    fn worker_artifact(
        worker_id: &str,
        sequence: usize,
        kind: ArtifactKind,
        summary: &str,
        content: &str,
    ) -> ArtifactEnvelope {
        ArtifactEnvelope::text(
            TraceArtifactId::new(format!("{worker_id}-artifact-{sequence}")).expect("artifact"),
            kind,
            summary,
            content,
            1_000,
        )
    }
}
