use crate::domain::model::{
    ExecutionPolicy, ExecutionPolicyDecision, ExecutionPolicyDecisionKind,
    ExecutionPolicyEvaluationInput, ExecutionPolicyMatcher, ExecutionPolicyRule,
    default_local_execution_policy,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionPolicyDecisionFixture {
    pub name: String,
    pub policy: ExecutionPolicy,
    pub input: ExecutionPolicyEvaluationInput,
    pub expected_decision: ExecutionPolicyDecisionKind,
    pub expected_rule_id: Option<String>,
}

impl ExecutionPolicyDecisionFixture {
    fn new(
        name: impl Into<String>,
        policy: ExecutionPolicy,
        input: ExecutionPolicyEvaluationInput,
        expected_decision: ExecutionPolicyDecisionKind,
        expected_rule_id: Option<&str>,
    ) -> Self {
        Self {
            name: name.into(),
            policy,
            input,
            expected_decision,
            expected_rule_id: expected_rule_id.map(str::to_string),
        }
    }
}

pub fn representative_execution_policy_fixtures() -> Vec<ExecutionPolicyDecisionFixture> {
    vec![
        ExecutionPolicyDecisionFixture::new(
            "default shell allow",
            default_local_execution_policy(),
            ExecutionPolicyEvaluationInput::command_for_tool("shell", ["git", "status"]),
            ExecutionPolicyDecisionKind::Allow,
            Some("allow-shell"),
        ),
        ExecutionPolicyDecisionFixture::new(
            "dangerous command deny",
            default_local_execution_policy(),
            ExecutionPolicyEvaluationInput::command_for_tool("shell", ["rm", "-rf", "/"]),
            ExecutionPolicyDecisionKind::Deny,
            Some("deny-root-removal"),
        ),
        ExecutionPolicyDecisionFixture::new(
            "publish prompt",
            ExecutionPolicy::new(vec![ExecutionPolicyRule::new(
                "prompt-cargo-publish",
                ExecutionPolicyMatcher::command_prefix(["cargo", "publish"]),
                ExecutionPolicyDecisionKind::Prompt,
                "publishing artifacts requires explicit operator review",
            )]),
            ExecutionPolicyEvaluationInput::command_for_tool("shell", ["cargo", "publish"]),
            ExecutionPolicyDecisionKind::Prompt,
            Some("prompt-cargo-publish"),
        ),
        ExecutionPolicyDecisionFixture::new(
            "test retry on failure",
            ExecutionPolicy::new(vec![ExecutionPolicyRule::new(
                "retry-cargo-test",
                ExecutionPolicyMatcher::command_prefix(["cargo", "test"]),
                ExecutionPolicyDecisionKind::OnFailure,
                "test failures may trigger one bounded recovery attempt",
            )]),
            ExecutionPolicyEvaluationInput::command_for_tool("shell", ["cargo", "test"]),
            ExecutionPolicyDecisionKind::OnFailure,
            Some("retry-cargo-test"),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::{ExecutionPolicyEvaluator, representative_execution_policy_fixtures};
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

    #[test]
    fn execution_policy_fixtures_document_representative_command_decisions() {
        let fixtures = representative_execution_policy_fixtures();

        assert_eq!(fixtures.len(), 4);
        for fixture in fixtures {
            let decision = ExecutionPolicyEvaluator::evaluate(&fixture.policy, &fixture.input);

            assert_eq!(
                decision.kind, fixture.expected_decision,
                "fixture `{}` should keep its documented decision",
                fixture.name
            );
            assert_eq!(
                decision.rule_id.as_deref(),
                fixture.expected_rule_id.as_deref(),
                "fixture `{}` should keep its documented policy rule",
                fixture.name
            );
        }
    }
}
