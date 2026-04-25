use super::{EvalRunner, PreparedRuntimeLanes, recursive_harness_eval_corpus};
use crate::domain::model::{
    EvalReport, EvalRunConfig, EvalStatus, ExecutionGovernanceProfile, ExecutionPolicy,
    ExternalCapabilityDescriptor, default_local_execution_policy,
};
use crate::domain::ports::{ProviderRegistryPosture, ProviderRegistryPostureRequest};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HarnessCapabilityRuntimeStatus {
    pub external_capabilities: Vec<HarnessExternalCapabilityRuntimeStatus>,
    pub execution_policy: HarnessExecutionPolicyRuntimeStatus,
    pub evals: HarnessEvalRuntimeStatus,
    pub provider_registry: HarnessProviderRegistryRuntimeStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HarnessExternalCapabilityRuntimeStatus {
    pub id: String,
    pub kind: String,
    pub availability: String,
    pub auth: String,
    pub effects: String,
    pub hand: String,
    pub required_permissions: Vec<String>,
    pub evidence_kinds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HarnessExecutionPolicyRuntimeStatus {
    pub profile_id: String,
    pub sandbox_mode: String,
    pub approval_policy: String,
    pub supported_reuse_scopes: Vec<String>,
    pub allowed_permissions: Vec<String>,
    pub default_decision: String,
    pub rules: Vec<HarnessExecutionPolicyRuleRuntimeStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HarnessExecutionPolicyRuleRuntimeStatus {
    pub id: String,
    pub decision: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HarnessEvalRuntimeStatus {
    pub offline: bool,
    pub scenario_count: usize,
    pub passed_reports: usize,
    pub failed_reports: usize,
    pub scenario_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HarnessProviderRegistryRuntimeStatus {
    pub network_discovery_required: bool,
    pub offline_safe: bool,
    pub entries: Vec<HarnessProviderModelRuntimeStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HarnessProviderModelRuntimeStatus {
    pub provider: String,
    pub model_id: String,
    pub status: String,
    pub reason: Option<String>,
}

pub struct RuntimeHarnessCapabilityPostureService;

impl RuntimeHarnessCapabilityPostureService {
    pub fn project(
        prepared_lanes: &PreparedRuntimeLanes,
        external_capabilities: &[ExternalCapabilityDescriptor],
    ) -> HarnessCapabilityRuntimeStatus {
        let profile = prepared_lanes.harness_profile();
        let execution_policy = default_local_execution_policy();
        let eval_config = EvalRunConfig::default();
        let eval_reports = EvalRunner::new(recursive_harness_eval_corpus()).run(&eval_config);
        let provider_posture = provider_registry_posture_for_prepared_lanes(prepared_lanes);

        HarnessCapabilityRuntimeStatus {
            external_capabilities: external_capabilities
                .iter()
                .map(HarnessExternalCapabilityRuntimeStatus::from_descriptor)
                .collect(),
            execution_policy: HarnessExecutionPolicyRuntimeStatus::from_policy(
                profile.active_profile_id(),
                profile.active_execution_governance(),
                &execution_policy,
            ),
            evals: HarnessEvalRuntimeStatus::from_reports(eval_config, &eval_reports),
            provider_registry: HarnessProviderRegistryRuntimeStatus::from_posture(
                &provider_posture,
            ),
        }
    }
}

fn provider_registry_posture_for_prepared_lanes(
    prepared_lanes: &PreparedRuntimeLanes,
) -> ProviderRegistryPosture {
    let mut configured = BTreeSet::new();
    configured.insert((
        prepared_lanes.planner.provider.name().to_string(),
        prepared_lanes.planner.model_id.clone(),
    ));
    configured.insert((
        prepared_lanes.synthesizer.provider.name().to_string(),
        prepared_lanes.synthesizer.model_id.clone(),
    ));

    ProviderRegistryPosture::from_configured_models(
        ProviderRegistryPostureRequest::local_first(),
        configured,
    )
}

impl HarnessExternalCapabilityRuntimeStatus {
    fn from_descriptor(descriptor: &ExternalCapabilityDescriptor) -> Self {
        Self {
            id: descriptor.id.clone(),
            kind: descriptor.kind.label().to_string(),
            availability: descriptor.availability.label().to_string(),
            auth: descriptor.auth_posture.label().to_string(),
            effects: descriptor.side_effect_posture.label().to_string(),
            hand: descriptor.hand.label().to_string(),
            required_permissions: descriptor
                .required_permissions
                .iter()
                .map(|permission| permission.label().to_string())
                .collect(),
            evidence_kinds: descriptor
                .evidence_shape
                .kinds
                .iter()
                .map(|kind| kind.label().to_string())
                .collect(),
        }
    }
}

impl HarnessExecutionPolicyRuntimeStatus {
    fn from_policy(
        profile_id: &str,
        governance: &ExecutionGovernanceProfile,
        policy: &ExecutionPolicy,
    ) -> Self {
        Self {
            profile_id: profile_id.to_string(),
            sandbox_mode: governance.sandbox_mode.label().to_string(),
            approval_policy: governance.approval_policy.label().to_string(),
            supported_reuse_scopes: governance
                .supported_reuse_scopes
                .iter()
                .map(|scope| scope.label().to_string())
                .collect(),
            allowed_permissions: governance
                .allowed_permissions
                .iter()
                .map(|permission| permission.label().to_string())
                .collect(),
            default_decision: policy.default_decision().label().to_string(),
            rules: policy
                .rules()
                .iter()
                .map(|rule| HarnessExecutionPolicyRuleRuntimeStatus {
                    id: rule.id.clone(),
                    decision: rule.decision.label().to_string(),
                    reason: rule.reason.clone(),
                })
                .collect(),
        }
    }
}

impl HarnessEvalRuntimeStatus {
    fn from_reports(config: EvalRunConfig, reports: &[EvalReport]) -> Self {
        Self {
            offline: config.offline,
            scenario_count: reports.len(),
            passed_reports: reports
                .iter()
                .filter(|report| report.status == EvalStatus::Passed)
                .count(),
            failed_reports: reports
                .iter()
                .filter(|report| report.status == EvalStatus::Failed)
                .count(),
            scenario_ids: reports
                .iter()
                .map(|report| report.scenario_id.clone())
                .collect(),
        }
    }
}

impl HarnessProviderRegistryRuntimeStatus {
    fn from_posture(posture: &ProviderRegistryPosture) -> Self {
        Self {
            network_discovery_required: posture.network_discovery_required,
            offline_safe: posture.is_offline_safe(),
            entries: posture
                .entries
                .iter()
                .map(|entry| HarnessProviderModelRuntimeStatus {
                    provider: entry.provider.clone(),
                    model_id: entry.model_id.clone(),
                    status: entry.status.label().to_string(),
                    reason: entry.reason.clone(),
                })
                .collect(),
        }
    }
}
