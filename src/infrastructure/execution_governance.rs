use crate::application::ExecutionPolicyEvaluator;
use crate::domain::model::{
    ExecutionApprovalPolicy, ExecutionEscalationRequest, ExecutionGovernanceOutcome,
    ExecutionGovernanceProfile, ExecutionPermission, ExecutionPermissionRequest,
    ExecutionPermissionReuseScope, ExecutionPolicy, ExecutionPolicyDecision,
    ExecutionPolicyDecisionKind, ExecutionPolicyEvaluationInput,
};
use std::process::Output;

#[derive(Debug)]
pub enum GovernedTerminalCommandResult {
    Executed {
        output: Output,
        governance_request: ExecutionPermissionRequest,
        governance_outcome: ExecutionGovernanceOutcome,
    },
    Blocked {
        governance_request: ExecutionPermissionRequest,
        governance_outcome: ExecutionGovernanceOutcome,
    },
}

pub struct ExecutionPermissionGate;

impl ExecutionPermissionGate {
    pub fn evaluate(
        profile: Option<&ExecutionGovernanceProfile>,
        request: &ExecutionPermissionRequest,
    ) -> ExecutionGovernanceOutcome {
        let Some(profile) = profile else {
            return ExecutionGovernanceOutcome::policy_unavailable(
                "active execution-governance profile is unavailable; failing closed",
                request.requirement.clone(),
            );
        };

        let missing_permissions = request
            .requirement
            .permissions
            .iter()
            .copied()
            .filter(|permission| !profile.permits(*permission))
            .collect::<Vec<_>>();
        if missing_permissions.is_empty() {
            return ExecutionGovernanceOutcome::allowed(
                format!(
                    "{} allowed under `{}` sandbox",
                    request.hand.label(),
                    profile.sandbox_mode.label()
                ),
                request.requirement.clone(),
                request.requirement.permissions.clone(),
            );
        }

        match profile.approval_policy {
            ExecutionApprovalPolicy::OnRequest => {
                let resolved_reuse_scope = request
                    .requested_reuse_scope
                    .filter(|scope| profile.supports_reuse_scope(*scope));
                let resolved_command_prefix =
                    if resolved_reuse_scope == Some(ExecutionPermissionReuseScope::CommandPrefix) {
                        request.requested_command_prefix.clone()
                    } else {
                        None
                    };
                let escalation = ExecutionEscalationRequest::new(
                    format!(
                        "{} requires approval for {}",
                        request.hand.label(),
                        request.requirement.summary
                    ),
                    missing_permissions,
                    resolved_reuse_scope,
                    resolved_command_prefix,
                );
                ExecutionGovernanceOutcome::escalation_required(
                    format!(
                        "{} exceeded the active `{}` sandbox and requires approval",
                        request.hand.label(),
                        profile.sandbox_mode.label()
                    ),
                    request.requirement.clone(),
                    escalation,
                )
            }
            ExecutionApprovalPolicy::Never => ExecutionGovernanceOutcome::denied(
                format!(
                    "{} exceeded the active `{}` sandbox and the approval policy is `never`",
                    request.hand.label(),
                    profile.sandbox_mode.label()
                ),
                request.requirement.clone(),
            ),
            ExecutionApprovalPolicy::OnFailure => ExecutionGovernanceOutcome::denied(
                format!(
                    "{} exceeded the active `{}` sandbox and `on_failure` cannot widen authority before execution",
                    request.hand.label(),
                    profile.sandbox_mode.label()
                ),
                request.requirement.clone(),
            ),
        }
    }
}

pub struct ExecutionPolicyPermissionGate;

impl ExecutionPolicyPermissionGate {
    pub fn evaluate(
        policy: Option<&ExecutionPolicy>,
        profile: Option<&ExecutionGovernanceProfile>,
        request: &ExecutionPermissionRequest,
        input: &ExecutionPolicyEvaluationInput,
    ) -> ExecutionGovernanceOutcome {
        let Some(policy) = policy else {
            return ExecutionGovernanceOutcome::policy_unavailable(
                "active execution policy is unavailable; failing closed",
                request.requirement.clone(),
            );
        };

        if let Some(error) = policy.validation_error() {
            return ExecutionGovernanceOutcome::policy_unavailable(
                format!("active execution policy is invalid: {error}; failing closed"),
                request.requirement.clone(),
            );
        }

        let decision = ExecutionPolicyEvaluator::evaluate(policy, input);
        match decision.kind {
            ExecutionPolicyDecisionKind::Allow | ExecutionPolicyDecisionKind::OnFailure => {
                let mut outcome = ExecutionPermissionGate::evaluate(profile, request);
                if matches!(
                    outcome.kind,
                    crate::domain::model::ExecutionGovernanceOutcomeKind::Allowed
                ) {
                    outcome.reason = format!(
                        "{}; {}",
                        summarize_policy_decision(&decision),
                        outcome.reason
                    );
                }
                outcome
            }
            ExecutionPolicyDecisionKind::Deny => ExecutionGovernanceOutcome::denied(
                summarize_policy_decision(&decision),
                request.requirement.clone(),
            ),
            ExecutionPolicyDecisionKind::Prompt => {
                prompt_required_outcome(profile, request, &decision)
            }
        }
    }
}

fn summarize_policy_decision(decision: &ExecutionPolicyDecision) -> String {
    match decision.rule_id.as_deref() {
        Some(rule_id) => format!(
            "execution policy {} via `{}`: {}",
            decision.kind.label(),
            rule_id,
            decision.reason
        ),
        None => format!(
            "execution policy {} by default: {}",
            decision.kind.label(),
            decision.reason
        ),
    }
}

fn prompt_required_outcome(
    profile: Option<&ExecutionGovernanceProfile>,
    request: &ExecutionPermissionRequest,
    decision: &ExecutionPolicyDecision,
) -> ExecutionGovernanceOutcome {
    match profile.map(|profile| profile.approval_policy) {
        Some(ExecutionApprovalPolicy::OnRequest) => {
            let resolved_reuse_scope = request.requested_reuse_scope.filter(|scope| {
                profile.is_some_and(|profile| profile.supports_reuse_scope(*scope))
            });
            let resolved_command_prefix =
                if resolved_reuse_scope == Some(ExecutionPermissionReuseScope::CommandPrefix) {
                    request.requested_command_prefix.clone()
                } else {
                    None
                };
            ExecutionGovernanceOutcome::escalation_required(
                summarize_policy_decision(decision),
                request.requirement.clone(),
                ExecutionEscalationRequest::new(
                    summarize_policy_decision(decision),
                    request.requirement.permissions.clone(),
                    resolved_reuse_scope,
                    resolved_command_prefix,
                ),
            )
        }
        Some(ExecutionApprovalPolicy::Never) => ExecutionGovernanceOutcome::denied(
            format!(
                "{}; active approval policy is `never`",
                summarize_policy_decision(decision)
            ),
            request.requirement.clone(),
        ),
        Some(ExecutionApprovalPolicy::OnFailure) => ExecutionGovernanceOutcome::denied(
            format!(
                "{}; `on_failure` cannot widen authority before execution",
                summarize_policy_decision(decision)
            ),
            request.requirement.clone(),
        ),
        None => ExecutionGovernanceOutcome::policy_unavailable(
            "active execution-governance profile is unavailable; execution policy prompt cannot be resolved; failing closed",
            request.requirement.clone(),
        ),
    }
}

pub fn summarize_governance_outcome(outcome: &ExecutionGovernanceOutcome) -> String {
    let mut summary = format!(
        "Execution governance {}: {}",
        outcome.kind.label(),
        outcome.reason
    );
    if let Some(escalation) = outcome.escalation_request.as_ref() {
        summary.push_str("\nRequested permissions: ");
        summary.push_str(
            &escalation
                .requested_permissions
                .iter()
                .map(|permission| permission.label())
                .collect::<Vec<_>>()
                .join(", "),
        );
        if let Some(scope) = escalation.reuse_scope {
            summary.push_str("\nReuse scope: ");
            summary.push_str(scope.label());
        }
        if let Some(prefix) = escalation
            .command_prefix
            .as_ref()
            .filter(|prefix| !prefix.is_empty())
        {
            summary.push_str("\nCommand prefix: ");
            summary.push_str(&prefix.join(" "));
        }
    }
    summary
}

pub fn terminal_command_permission_request(
    command: &str,
    tool_name: &str,
) -> ExecutionPermissionRequest {
    let requirement = crate::domain::model::ExecutionPermissionRequirement::new(
        format!("run `{tool_name}` command `{command}`"),
        vec![ExecutionPermission::RunWorkspaceCommand],
    );
    let request = ExecutionPermissionRequest::new(
        crate::domain::model::ExecutionHandKind::TerminalRunner,
        requirement,
    );
    let command_prefix = command
        .split_whitespace()
        .take(4)
        .map(str::to_string)
        .collect::<Vec<_>>();
    if command_prefix.is_empty() {
        request
    } else {
        request.with_bounded_reuse(ExecutionPermissionReuseScope::CommandPrefix, command_prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::{ExecutionPermissionGate, ExecutionPolicyPermissionGate};
    use crate::domain::model::{
        ExecutionApprovalPolicy, ExecutionGovernanceOutcomeKind, ExecutionGovernanceProfile,
        ExecutionHandKind, ExecutionPermission, ExecutionPermissionRequest,
        ExecutionPermissionRequirement, ExecutionPermissionReuseScope, ExecutionPolicy,
        ExecutionPolicyDecisionKind, ExecutionPolicyEvaluationInput, ExecutionPolicyMatcher,
        ExecutionPolicyRule, ExecutionSandboxMode, default_local_execution_governance_profile,
    };

    #[test]
    fn gate_escalates_with_bounded_command_prefix_reuse_when_profile_allows_it() {
        let profile = ExecutionGovernanceProfile::new(
            ExecutionSandboxMode::ReadOnly,
            ExecutionApprovalPolicy::OnRequest,
            vec![ExecutionPermissionReuseScope::CommandPrefix],
            None,
        );
        let request = ExecutionPermissionRequest::new(
            ExecutionHandKind::TerminalRunner,
            ExecutionPermissionRequirement::new(
                "run `cargo test`",
                vec![ExecutionPermission::RunWorkspaceCommand],
            ),
        )
        .with_bounded_reuse(
            ExecutionPermissionReuseScope::CommandPrefix,
            vec!["cargo".to_string(), "test".to_string()],
        );

        let outcome = ExecutionPermissionGate::evaluate(Some(&profile), &request);

        assert_eq!(
            outcome.kind,
            ExecutionGovernanceOutcomeKind::EscalationRequired
        );
        let escalation = outcome.escalation_request.expect("escalation request");
        assert_eq!(
            escalation.requested_permissions,
            vec![ExecutionPermission::RunWorkspaceCommand]
        );
        assert_eq!(
            escalation.reuse_scope,
            Some(ExecutionPermissionReuseScope::CommandPrefix)
        );
        assert_eq!(
            escalation.command_prefix,
            Some(vec!["cargo".to_string(), "test".to_string()])
        );
    }

    #[test]
    fn gate_fails_closed_when_the_active_profile_is_missing() {
        let request = ExecutionPermissionRequest::new(
            ExecutionHandKind::WorkspaceEditor,
            ExecutionPermissionRequirement::new(
                "write `notes.txt`",
                vec![
                    ExecutionPermission::ReadWorkspace,
                    ExecutionPermission::WriteWorkspace,
                ],
            ),
        );

        let outcome = ExecutionPermissionGate::evaluate(None, &request);

        assert_eq!(
            outcome.kind,
            ExecutionGovernanceOutcomeKind::PolicyUnavailable
        );
        assert!(
            outcome
                .reason
                .contains("active execution-governance profile")
        );
    }

    #[test]
    fn execution_policy_defaults_fail_closed_when_policy_is_unavailable_or_invalid() {
        let profile = default_local_execution_governance_profile();
        let request = super::terminal_command_permission_request("true", "shell");
        let input = ExecutionPolicyEvaluationInput::command_for_tool("shell", ["true"]);

        let unavailable =
            ExecutionPolicyPermissionGate::evaluate(None, Some(&profile), &request, &input);

        assert_eq!(
            unavailable.kind,
            ExecutionGovernanceOutcomeKind::PolicyUnavailable
        );
        assert!(unavailable.reason.contains("execution policy"));

        let invalid_policy = ExecutionPolicy::new(vec![ExecutionPolicyRule::new(
            "",
            ExecutionPolicyMatcher::tool("shell"),
            ExecutionPolicyDecisionKind::Allow,
            "rule ids must be present",
        )]);
        let invalid = ExecutionPolicyPermissionGate::evaluate(
            Some(&invalid_policy),
            Some(&profile),
            &request,
            &input,
        );

        assert_eq!(
            invalid.kind,
            ExecutionGovernanceOutcomeKind::PolicyUnavailable
        );
        assert!(invalid.reason.contains("invalid"));
    }
}
