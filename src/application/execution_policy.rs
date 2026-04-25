use crate::domain::model::{
    ExecutionPolicy, ExecutionPolicyDecision, ExecutionPolicyEvaluationInput,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct ExecutionPolicyEvaluator;

impl ExecutionPolicyEvaluator {
    pub fn evaluate(
        policy: &ExecutionPolicy,
        input: &ExecutionPolicyEvaluationInput,
    ) -> ExecutionPolicyDecision {
        let mut best_match = None;

        for (index, rule) in policy.rules().iter().enumerate() {
            let Some(rank) = rule.match_rank(input) else {
                continue;
            };
            if best_match
                .as_ref()
                .is_none_or(|(best_rank, best_index, _)| {
                    rank > *best_rank || (rank == *best_rank && index < *best_index)
                })
            {
                best_match = Some((rank, index, rule));
            }
        }

        best_match
            .map(|(_, _, rule)| ExecutionPolicyDecision::from_rule(rule))
            .unwrap_or_else(|| ExecutionPolicyDecision::default(policy.default_decision()))
    }
}

#[cfg(test)]
mod tests {
    use super::ExecutionPolicyEvaluator;
    use crate::domain::model::{
        ExecutionPolicy, ExecutionPolicyDecisionKind, ExecutionPolicyEvaluationInput,
        ExecutionPolicyMatcher, ExecutionPolicyRule,
    };

    fn fixture_policy() -> ExecutionPolicy {
        ExecutionPolicy::new(vec![
            ExecutionPolicyRule::new(
                "prompt-cargo",
                ExecutionPolicyMatcher::executable("cargo"),
                ExecutionPolicyDecisionKind::Prompt,
                "broad cargo commands need review",
            ),
            ExecutionPolicyRule::new(
                "allow-cargo-test",
                ExecutionPolicyMatcher::command_prefix(["cargo", "test"]),
                ExecutionPolicyDecisionKind::Allow,
                "tests are a safe repository verification command",
            ),
            ExecutionPolicyRule::new(
                "deny-rm",
                ExecutionPolicyMatcher::command_prefix(["rm"]),
                ExecutionPolicyDecisionKind::Deny,
                "removal commands are denied",
            ),
            ExecutionPolicyRule::new(
                "on-failure-external",
                ExecutionPolicyMatcher::tool("external_capability"),
                ExecutionPolicyDecisionKind::OnFailure,
                "external fabrics may degrade and retry with typed evidence",
            ),
        ])
    }

    #[test]
    fn execution_policy_evaluator_uses_deterministic_prefix_and_executable_matching() {
        let policy = fixture_policy();

        let cargo_test = ExecutionPolicyEvaluator::evaluate(
            &policy,
            &ExecutionPolicyEvaluationInput::command(["cargo", "test", "--all-targets"]),
        );
        let cargo_fmt = ExecutionPolicyEvaluator::evaluate(
            &policy,
            &ExecutionPolicyEvaluationInput::command(["cargo", "fmt", "--all"]),
        );
        let rm = ExecutionPolicyEvaluator::evaluate(
            &policy,
            &ExecutionPolicyEvaluationInput::command(["rm", "-rf", "target"]),
        );
        let external = ExecutionPolicyEvaluator::evaluate(
            &policy,
            &ExecutionPolicyEvaluationInput::tool("external_capability"),
        );

        assert_eq!(cargo_test.kind, ExecutionPolicyDecisionKind::Allow);
        assert_eq!(cargo_test.rule_id.as_deref(), Some("allow-cargo-test"));
        assert_eq!(cargo_fmt.kind, ExecutionPolicyDecisionKind::Prompt);
        assert_eq!(cargo_fmt.rule_id.as_deref(), Some("prompt-cargo"));
        assert_eq!(rm.kind, ExecutionPolicyDecisionKind::Deny);
        assert_eq!(external.kind, ExecutionPolicyDecisionKind::OnFailure);
    }
}
