use crate::domain::model::CompactionBudget;
use crate::domain::ports::RefinementPolicy;
use crate::infrastructure::providers::{ModelCapabilitySurface, PlannerToolCallCapability};
use crate::infrastructure::rendering::RenderCapability;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HarnessProfileId {
    RecursiveStructuredV1,
    PromptEnvelopeSafeV1,
}

impl HarnessProfileId {
    pub fn id(self) -> &'static str {
        match self {
            Self::RecursiveStructuredV1 => "recursive-structured-v1",
            Self::PromptEnvelopeSafeV1 => "prompt-envelope-safe-v1",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HarnessSteeringPolicy {
    pub refinement_policy_id: &'static str,
    pub max_refinements_per_turn: usize,
    pub cooldown_steps: usize,
    pub oscillation_signature_window: usize,
    pub signature_history_limit: usize,
}

impl HarnessSteeringPolicy {
    pub fn into_refinement_policy(&self) -> RefinementPolicy {
        RefinementPolicy {
            id: self.refinement_policy_id.to_string(),
            enabled: true,
            trigger: Default::default(),
            max_refinements_per_turn: self.max_refinements_per_turn,
            cooldown_steps: self.cooldown_steps,
            oscillation_signature_window: self.oscillation_signature_window,
            signature_history_limit: self.signature_history_limit,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HarnessCompactionPolicy {
    pub profile_id: &'static str,
    pub budget: CompactionBudget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HarnessRecoveryPolicy {
    pub mode: &'static str,
    pub invalid_reply_retries: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HarnessProfile {
    pub id: HarnessProfileId,
    pub steering: HarnessSteeringPolicy,
    pub compaction: HarnessCompactionPolicy,
    pub recovery: HarnessRecoveryPolicy,
    pub specialist_brain_ids: &'static [&'static str],
}

impl HarnessProfile {
    pub fn id(&self) -> &'static str {
        self.id.id()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HarnessProfileSelection {
    pub requested: HarnessProfile,
    pub active: HarnessProfile,
    pub downgrade_reason: Option<String>,
}

impl HarnessProfileSelection {
    pub fn resolve(
        planner_surface: &ModelCapabilitySurface,
        synthesizer_surface: &ModelCapabilitySurface,
    ) -> Self {
        let requested = recursive_structured_v1();
        let mut downgrade_reasons = Vec::new();

        if matches!(
            planner_surface.planner_tool_call,
            PlannerToolCallCapability::PromptEnvelope
        ) {
            downgrade_reasons
                .push("planner next-action transport requires prompt-envelope recovery");
        }
        if matches!(
            synthesizer_surface.render_capability,
            RenderCapability::PromptEnvelope
        ) {
            downgrade_reasons.push("final-answer transport requires prompt-envelope rendering");
        }

        let downgrade_reason = if downgrade_reasons.is_empty() {
            None
        } else {
            Some(downgrade_reasons.join("; "))
        };
        let active = if downgrade_reason.is_some() {
            prompt_envelope_safe_v1()
        } else {
            requested.clone()
        };

        Self {
            requested,
            active,
            downgrade_reason,
        }
    }

    pub fn active_profile_id(&self) -> &'static str {
        self.active.id()
    }

    pub fn active_refinement_policy(&self) -> RefinementPolicy {
        self.active.steering.into_refinement_policy()
    }

    pub fn active_compaction_budget(&self) -> CompactionBudget {
        self.active.compaction.budget.clone()
    }

    pub fn active_specialist_brain_ids(&self) -> &'static [&'static str] {
        self.active.specialist_brain_ids
    }
}

fn recursive_structured_v1() -> HarnessProfile {
    HarnessProfile {
        id: HarnessProfileId::RecursiveStructuredV1,
        steering: HarnessSteeringPolicy {
            refinement_policy_id: "policy:context-refine-structured-v1",
            max_refinements_per_turn: 1,
            cooldown_steps: 2,
            oscillation_signature_window: 3,
            signature_history_limit: 6,
        },
        compaction: HarnessCompactionPolicy {
            profile_id: "compaction:structured-v1",
            budget: CompactionBudget { max_steps: 3 },
        },
        recovery: HarnessRecoveryPolicy {
            mode: "structured-retry",
            invalid_reply_retries: 1,
        },
        specialist_brain_ids: &["session-continuity-v1"],
    }
}

fn prompt_envelope_safe_v1() -> HarnessProfile {
    HarnessProfile {
        id: HarnessProfileId::PromptEnvelopeSafeV1,
        steering: HarnessSteeringPolicy {
            refinement_policy_id: "policy:context-refine-prompt-envelope-v1",
            max_refinements_per_turn: 1,
            cooldown_steps: 3,
            oscillation_signature_window: 2,
            signature_history_limit: 4,
        },
        compaction: HarnessCompactionPolicy {
            profile_id: "compaction:prompt-envelope-v1",
            budget: CompactionBudget { max_steps: 2 },
        },
        recovery: HarnessRecoveryPolicy {
            mode: "prompt-envelope-retry",
            invalid_reply_retries: 2,
        },
        specialist_brain_ids: &["session-continuity-v1"],
    }
}

#[cfg(test)]
mod tests {
    use super::HarnessProfileSelection;
    use crate::infrastructure::providers::ModelProvider;

    #[test]
    fn structured_capability_surfaces_keep_the_structured_profile_active() {
        let selection = HarnessProfileSelection::resolve(
            &ModelProvider::Google.capability_surface("gemini-2.5-flash"),
            &ModelProvider::Openai.capability_surface("gpt-5.4"),
        );

        assert_eq!(selection.requested.id(), "recursive-structured-v1");
        assert_eq!(selection.active.id(), "recursive-structured-v1");
        assert_eq!(selection.downgrade_reason, None);
        assert_eq!(
            selection.active_refinement_policy().id,
            "policy:context-refine-structured-v1"
        );
        assert_eq!(selection.active_compaction_budget().max_steps, 3);
    }

    #[test]
    fn prompt_envelope_capabilities_explicitly_downgrade_the_active_profile() {
        let selection = HarnessProfileSelection::resolve(
            &ModelProvider::Anthropic.capability_surface("claude-sonnet-4-20250514"),
            &ModelProvider::Sift.capability_surface("qwen-1.5b"),
        );

        assert_eq!(selection.requested.id(), "recursive-structured-v1");
        assert_eq!(selection.active.id(), "prompt-envelope-safe-v1");
        assert_eq!(
            selection.downgrade_reason.as_deref(),
            Some(
                "planner next-action transport requires prompt-envelope recovery; final-answer transport requires prompt-envelope rendering"
            )
        );
        assert_eq!(
            selection.active_refinement_policy().id,
            "policy:context-refine-prompt-envelope-v1"
        );
        assert_eq!(selection.active_compaction_budget().max_steps, 2);
    }
}
