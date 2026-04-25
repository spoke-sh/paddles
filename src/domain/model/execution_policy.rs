use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPolicyDecisionKind {
    Allow,
    Prompt,
    Deny,
    OnFailure,
}

impl ExecutionPolicyDecisionKind {
    pub const ALL: [Self; 4] = [Self::Allow, Self::Prompt, Self::Deny, Self::OnFailure];

    pub fn label(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Prompt => "prompt",
            Self::Deny => "deny",
            Self::OnFailure => "on_failure",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ExecutionPolicyMatcher {
    CommandPrefix(Vec<String>),
    Executable(String),
    Tool(String),
}

impl ExecutionPolicyMatcher {
    pub fn command_prefix(tokens: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::CommandPrefix(tokens.into_iter().map(Into::into).collect())
    }

    pub fn executable(executable: impl Into<String>) -> Self {
        Self::Executable(executable.into())
    }

    pub fn tool(tool_name: impl Into<String>) -> Self {
        Self::Tool(tool_name.into())
    }

    fn match_rank(&self, input: &ExecutionPolicyEvaluationInput) -> Option<(u8, usize)> {
        match self {
            Self::CommandPrefix(prefix)
                if !prefix.is_empty() && command_starts_with(input.command_tokens(), prefix) =>
            {
                Some((3, prefix.len()))
            }
            Self::Executable(executable)
                if input
                    .executable()
                    .is_some_and(|candidate| candidate == executable) =>
            {
                Some((1, 1))
            }
            Self::Tool(tool_name)
                if input
                    .tool_name()
                    .is_some_and(|candidate| candidate == tool_name) =>
            {
                Some((2, 1))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPolicyRule {
    pub id: String,
    pub matcher: ExecutionPolicyMatcher,
    pub decision: ExecutionPolicyDecisionKind,
    pub reason: String,
}

impl ExecutionPolicyRule {
    pub fn new(
        id: impl Into<String>,
        matcher: ExecutionPolicyMatcher,
        decision: ExecutionPolicyDecisionKind,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            matcher,
            decision,
            reason: reason.into(),
        }
    }

    pub fn match_rank(&self, input: &ExecutionPolicyEvaluationInput) -> Option<(u8, usize)> {
        self.matcher.match_rank(input)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPolicy {
    rules: Vec<ExecutionPolicyRule>,
    default_decision: ExecutionPolicyDecisionKind,
}

impl ExecutionPolicy {
    pub fn new(rules: Vec<ExecutionPolicyRule>) -> Self {
        Self {
            rules,
            default_decision: ExecutionPolicyDecisionKind::Deny,
        }
    }

    pub fn with_default_decision(mut self, decision: ExecutionPolicyDecisionKind) -> Self {
        self.default_decision = decision;
        self
    }

    pub fn rules(&self) -> &[ExecutionPolicyRule] {
        &self.rules
    }

    pub fn default_decision(&self) -> ExecutionPolicyDecisionKind {
        self.default_decision
    }

    pub fn validation_error(&self) -> Option<String> {
        self.rules.iter().find_map(|rule| {
            if rule.id.trim().is_empty() {
                return Some("execution policy rule id must not be empty".to_string());
            }
            if rule.reason.trim().is_empty() {
                return Some(format!(
                    "execution policy rule `{}` must include a reason",
                    rule.id
                ));
            }
            match &rule.matcher {
                ExecutionPolicyMatcher::CommandPrefix(prefix) if prefix.is_empty() => {
                    Some(format!(
                        "execution policy rule `{}` has an empty command prefix",
                        rule.id
                    ))
                }
                ExecutionPolicyMatcher::CommandPrefix(prefix)
                    if prefix.iter().any(|token| token.trim().is_empty()) =>
                {
                    Some(format!(
                        "execution policy rule `{}` has an empty command prefix token",
                        rule.id
                    ))
                }
                ExecutionPolicyMatcher::Executable(executable) if executable.trim().is_empty() => {
                    Some(format!(
                        "execution policy rule `{}` has an empty executable matcher",
                        rule.id
                    ))
                }
                ExecutionPolicyMatcher::Tool(tool) if tool.trim().is_empty() => Some(format!(
                    "execution policy rule `{}` has an empty tool matcher",
                    rule.id
                )),
                _ => None,
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPolicyEvaluationInput {
    command_tokens: Vec<String>,
    tool_name: Option<String>,
}

impl ExecutionPolicyEvaluationInput {
    pub fn command(tokens: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            command_tokens: tokens.into_iter().map(Into::into).collect(),
            tool_name: None,
        }
    }

    pub fn command_for_tool(
        tool_name: impl Into<String>,
        tokens: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            command_tokens: tokens.into_iter().map(Into::into).collect(),
            tool_name: Some(tool_name.into()),
        }
    }

    pub fn tool(tool_name: impl Into<String>) -> Self {
        Self {
            command_tokens: Vec::new(),
            tool_name: Some(tool_name.into()),
        }
    }

    pub fn command_tokens(&self) -> &[String] {
        &self.command_tokens
    }

    pub fn executable(&self) -> Option<&str> {
        self.command_tokens.first().map(String::as_str)
    }

    pub fn tool_name(&self) -> Option<&str> {
        self.tool_name.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPolicyDecision {
    pub kind: ExecutionPolicyDecisionKind,
    pub rule_id: Option<String>,
    pub reason: String,
}

impl ExecutionPolicyDecision {
    pub fn from_rule(rule: &ExecutionPolicyRule) -> Self {
        Self {
            kind: rule.decision,
            rule_id: Some(rule.id.clone()),
            reason: rule.reason.clone(),
        }
    }

    pub fn default(kind: ExecutionPolicyDecisionKind) -> Self {
        Self {
            kind,
            rule_id: None,
            reason: format!("no execution policy rule matched; default {}", kind.label()),
        }
    }
}

fn command_starts_with(command: &[String], prefix: &[String]) -> bool {
    command.len() >= prefix.len()
        && command
            .iter()
            .zip(prefix.iter())
            .all(|(command, prefix)| command == prefix)
}

pub fn default_local_execution_policy() -> ExecutionPolicy {
    ExecutionPolicy::new(vec![
        ExecutionPolicyRule::new(
            "deny-root-removal",
            ExecutionPolicyMatcher::command_prefix(["rm", "-rf", "/"]),
            ExecutionPolicyDecisionKind::Deny,
            "root removal is never a valid local harness operation",
        ),
        ExecutionPolicyRule::new(
            "allow-inspect",
            ExecutionPolicyMatcher::tool("inspect"),
            ExecutionPolicyDecisionKind::Allow,
            "single-step read-only inspection remains governed by the active sandbox",
        ),
        ExecutionPolicyRule::new(
            "allow-shell",
            ExecutionPolicyMatcher::tool("shell"),
            ExecutionPolicyDecisionKind::Allow,
            "workspace shell execution remains governed by the active sandbox",
        ),
        ExecutionPolicyRule::new(
            "allow-diff",
            ExecutionPolicyMatcher::tool("diff"),
            ExecutionPolicyDecisionKind::Allow,
            "workspace diff is a read-only local operation",
        ),
        ExecutionPolicyRule::new(
            "allow-write-file",
            ExecutionPolicyMatcher::tool("write_file"),
            ExecutionPolicyDecisionKind::Allow,
            "workspace writes remain governed by the active sandbox",
        ),
        ExecutionPolicyRule::new(
            "allow-replace-in-file",
            ExecutionPolicyMatcher::tool("replace_in_file"),
            ExecutionPolicyDecisionKind::Allow,
            "workspace replacements remain governed by the active sandbox",
        ),
        ExecutionPolicyRule::new(
            "allow-apply-patch",
            ExecutionPolicyMatcher::tool("apply_patch"),
            ExecutionPolicyDecisionKind::Allow,
            "workspace patches remain governed by the active sandbox",
        ),
        ExecutionPolicyRule::new(
            "allow-external-capability-through-governance",
            ExecutionPolicyMatcher::tool("external_capability"),
            ExecutionPolicyDecisionKind::Allow,
            "external capability calls remain governed by descriptor permissions",
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::{
        ExecutionPolicy, ExecutionPolicyDecisionKind, ExecutionPolicyMatcher, ExecutionPolicyRule,
    };

    #[test]
    fn execution_policy_rules_express_allow_prompt_deny_and_on_failure_decisions() {
        let policy = ExecutionPolicy::new(vec![
            ExecutionPolicyRule::new(
                "allow-cargo-test",
                ExecutionPolicyMatcher::command_prefix(["cargo", "test"]),
                ExecutionPolicyDecisionKind::Allow,
                "repository tests are allowed",
            ),
            ExecutionPolicyRule::new(
                "prompt-cargo",
                ExecutionPolicyMatcher::executable("cargo"),
                ExecutionPolicyDecisionKind::Prompt,
                "other cargo commands need operator review",
            ),
            ExecutionPolicyRule::new(
                "deny-rm",
                ExecutionPolicyMatcher::command_prefix(["rm"]),
                ExecutionPolicyDecisionKind::Deny,
                "destructive shell removal is denied",
            ),
            ExecutionPolicyRule::new(
                "retry-tests-on-failure",
                ExecutionPolicyMatcher::command_prefix(["cargo", "test"]),
                ExecutionPolicyDecisionKind::OnFailure,
                "test failures may trigger one bounded recovery attempt",
            ),
        ]);

        assert_eq!(policy.rules().len(), 4);
        assert_eq!(
            ExecutionPolicyDecisionKind::ALL.map(ExecutionPolicyDecisionKind::label),
            ["allow", "prompt", "deny", "on_failure"]
        );
        assert!(policy.rules().iter().any(|rule| {
            rule.decision == ExecutionPolicyDecisionKind::OnFailure
                && rule.reason.contains("bounded recovery")
        }));
    }
}
