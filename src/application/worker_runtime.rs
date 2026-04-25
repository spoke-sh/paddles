use crate::domain::model::{
    ConversationThreadRef, TraceBranchId, TraceWorkerLifecycle, WorkerDelegationContract,
    WorkerDelegationRequest, WorkerLifecycleOperation, WorkerLifecycleResult,
    WorkerLifecycleResultStatus,
};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerRuntimeSpawnRequest {
    pub instruction: String,
    pub contract: WorkerDelegationContract,
    pub parent_thread: ConversationThreadRef,
    pub budget: WorkerRuntimeBudget,
}

impl WorkerRuntimeSpawnRequest {
    pub fn new(instruction: impl Into<String>, contract: WorkerDelegationContract) -> Self {
        Self {
            instruction: instruction.into(),
            contract,
            parent_thread: ConversationThreadRef::Mainline,
            budget: WorkerRuntimeBudget::default(),
        }
    }

    pub fn with_parent_thread(mut self, parent_thread: ConversationThreadRef) -> Self {
        self.parent_thread = parent_thread;
        self
    }

    pub fn with_budget(mut self, budget: WorkerRuntimeBudget) -> Self {
        self.budget = budget;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerRuntimeSpawnOutcome {
    pub worker_id: String,
    pub worker_thread: ConversationThreadRef,
    pub lifecycle: TraceWorkerLifecycle,
    pub budget: WorkerRuntimeBudget,
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BoundedWorkerRuntime, WorkerRuntimeBudget, WorkerRuntimePort, WorkerRuntimeSpawnRequest,
    };
    use crate::domain::model::{
        DelegationEvidencePolicy, DelegationGovernancePolicy, DelegationIntegrationOwner,
        ExecutionApprovalPolicy, ExecutionGovernanceProfile, ExecutionGovernanceSnapshot,
        ExecutionPermissionReuseScope, ExecutionSandboxMode, TraceRecordKind, WorkerArtifactKind,
        WorkerDelegationContract, WorkerDelegationRequest, WorkerLifecycleOperation,
        WorkerLifecycleResultStatus, WorkerOwnership, WorkerRole,
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

    fn worker_contract(summary: &str) -> WorkerDelegationContract {
        WorkerDelegationContract::new(
            WorkerRole::new("worker", "Worker", "Run bounded delegated work."),
            WorkerOwnership::new(
                summary,
                vec!["src".to_string()],
                vec!["src/application".to_string()],
                DelegationIntegrationOwner::Parent,
            ),
            DelegationGovernancePolicy::inherit_from_parent(
                &ExecutionGovernanceSnapshot::new(
                    "test-profile",
                    "test-profile",
                    ExecutionGovernanceProfile::new(
                        ExecutionSandboxMode::WorkspaceWrite,
                        ExecutionApprovalPolicy::OnRequest,
                        vec![ExecutionPermissionReuseScope::Turn],
                        None,
                    ),
                ),
                DelegationEvidencePolicy::new(
                    "Worker evidence remains parent-visible.",
                    vec![
                        WorkerArtifactKind::ToolCall,
                        WorkerArtifactKind::ToolOutput,
                        WorkerArtifactKind::CompletionSummary,
                    ],
                ),
            ),
        )
    }
}
