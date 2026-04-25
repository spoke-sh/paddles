use crate::domain::model::InstructionFrame;
use crate::domain::ports::{PlannerBudget, PlannerLoopState};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct PlannerLoopService;

impl PlannerLoopService {
    pub(super) const fn new() -> Self {
        Self
    }

    pub(super) fn budget_for_replan_attempt(
        &self,
        base_budget: &PlannerBudget,
        completed_replans: usize,
    ) -> PlannerBudget {
        let multiplier = completed_replans.saturating_add(1);
        PlannerBudget {
            max_steps: base_budget.max_steps.saturating_mul(multiplier),
            max_branch_factor: base_budget.max_branch_factor,
            max_evidence_items: base_budget.max_evidence_items,
            max_reads: base_budget.max_reads.saturating_mul(multiplier),
            max_inspects: base_budget.max_inspects.saturating_mul(multiplier),
            max_searches: base_budget.max_searches.saturating_mul(multiplier),
            max_replans: base_budget.max_replans,
        }
    }

    pub(super) fn activate_replan(
        &self,
        stop_reason: &str,
        activation: PlannerLoopReplanActivation<'_>,
    ) -> Option<PlannerLoopReplanEvent> {
        let PlannerLoopReplanActivation {
            instruction_frame,
            base_budget,
            completed_replans,
            budget,
            loop_state,
        } = activation;

        if !self.should_activate_replan(
            stop_reason,
            instruction_frame,
            *completed_replans,
            base_budget,
        ) {
            return None;
        }

        *completed_replans += 1;
        *budget = self.budget_for_replan_attempt(base_budget, *completed_replans);
        self.sync_replan_note(loop_state, stop_reason, instruction_frame);

        Some(PlannerLoopReplanEvent {
            stage: "replan",
            reason: format!(
                "pending edit remained open after {}; extending planner budget to {} steps, {} reads, {} inspects, and {} searches while continuing from current evidence",
                Self::budget_stop_reason_label(stop_reason),
                budget.max_steps,
                budget.max_reads,
                budget.max_inspects,
                budget.max_searches,
            ),
        })
    }

    fn should_activate_replan(
        &self,
        stop_reason: &str,
        instruction_frame: Option<&InstructionFrame>,
        completed_replans: usize,
        base_budget: &PlannerBudget,
    ) -> bool {
        instruction_frame.is_some_and(InstructionFrame::has_pending_workspace_obligation)
            && completed_replans < base_budget.max_replans
            && Self::stop_reason_supports_replan(stop_reason)
    }

    fn sync_replan_note(
        &self,
        loop_state: &mut PlannerLoopState,
        stop_reason: &str,
        instruction_frame: Option<&InstructionFrame>,
    ) {
        const REPLAN_NOTE_PREFIX: &str = "Replan from current evidence";

        loop_state
            .notes
            .retain(|note| !note.starts_with(REPLAN_NOTE_PREFIX));

        let mut lines = vec![format!(
            "Replan from current evidence after {}.",
            Self::budget_stop_reason_label(stop_reason)
        )];
        lines.push(
            "Do not restart broad exploration or repeat missing or failed paths.".to_string(),
        );
        let next_step_line = match instruction_frame {
            Some(frame) if frame.requires_applied_edit() && frame.requires_applied_commit() => {
                "Choose the single most direct next step toward the requested workspace change and git commit."
            }
            Some(frame) if frame.requires_applied_commit() => {
                "Choose the single most direct next step toward recording the requested git commit."
            }
            _ => "Choose the single most direct next step toward an applied repository change.",
        };
        lines.push(next_step_line.to_string());
        if let Some(summary) = instruction_frame.and_then(InstructionFrame::candidate_summary) {
            lines.push(format!("Authored candidate files: {summary}"));
        }
        loop_state.notes.push(lines.join("\n"));
    }

    fn budget_stop_reason_label(stop_reason: &str) -> String {
        stop_reason
            .strip_suffix("-exhausted")
            .unwrap_or(stop_reason)
            .replace('-', " ")
    }

    fn stop_reason_supports_replan(stop_reason: &str) -> bool {
        stop_reason.contains("budget-exhausted")
            || stop_reason == "planner-budget-exhausted"
            || stop_reason == "instruction-unsatisfied"
    }
}

pub(super) struct PlannerLoopReplanActivation<'a> {
    pub(super) instruction_frame: Option<&'a InstructionFrame>,
    pub(super) base_budget: &'a PlannerBudget,
    pub(super) completed_replans: &'a mut usize,
    pub(super) budget: &'a mut PlannerBudget,
    pub(super) loop_state: &'a mut PlannerLoopState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlannerLoopReplanEvent {
    pub(super) stage: &'static str,
    pub(super) reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit_budget() -> PlannerBudget {
        PlannerBudget {
            max_steps: 4,
            max_branch_factor: 2,
            max_evidence_items: 8,
            max_reads: 2,
            max_inspects: 1,
            max_searches: 1,
            max_replans: 1,
        }
    }

    #[test]
    fn planner_loop_replan_extends_budget_and_keeps_current_evidence_context() {
        let service = PlannerLoopService::new();
        let base_budget = edit_budget();
        let mut completed_replans = 0;
        let mut budget = service.budget_for_replan_attempt(&base_budget, completed_replans);
        let mut loop_state = PlannerLoopState {
            notes: vec![
                "existing planner note".to_string(),
                "Replan from current evidence after prior attempt.".to_string(),
            ],
            ..PlannerLoopState::default()
        };
        let instruction_frame = InstructionFrame::for_edit(vec!["src/lib.rs".to_string()]);

        let event = service
            .activate_replan(
                "planner-budget-exhausted",
                PlannerLoopReplanActivation {
                    instruction_frame: Some(&instruction_frame),
                    base_budget: &base_budget,
                    completed_replans: &mut completed_replans,
                    budget: &mut budget,
                    loop_state: &mut loop_state,
                },
            )
            .expect("pending edit should activate one replan");

        assert_eq!(completed_replans, 1);
        assert_eq!(budget.max_steps, 8);
        assert_eq!(budget.max_reads, 4);
        assert_eq!(budget.max_inspects, 2);
        assert_eq!(budget.max_searches, 2);
        assert_eq!(event.stage, "replan");
        assert!(event.reason.contains("planner budget"));
        assert!(event.reason.contains("8 steps"));
        assert!(
            loop_state
                .notes
                .contains(&"existing planner note".to_string())
        );
        assert_eq!(
            loop_state
                .notes
                .iter()
                .filter(|note| note.starts_with("Replan from current evidence"))
                .count(),
            1
        );
        assert!(loop_state.notes.iter().any(|note| {
            note.contains("Do not restart broad exploration")
                && note.contains("Authored candidate files: src/lib.rs")
        }));
    }

    #[test]
    fn planner_loop_replan_does_not_activate_without_pending_workspace_obligation() {
        let service = PlannerLoopService::new();
        let base_budget = edit_budget();
        let mut completed_replans = 0;
        let mut budget = service.budget_for_replan_attempt(&base_budget, completed_replans);
        let mut loop_state = PlannerLoopState::default();

        let event = service.activate_replan(
            "planner-budget-exhausted",
            PlannerLoopReplanActivation {
                instruction_frame: None,
                base_budget: &base_budget,
                completed_replans: &mut completed_replans,
                budget: &mut budget,
                loop_state: &mut loop_state,
            },
        );

        assert!(event.is_none());
        assert_eq!(completed_replans, 0);
        assert_eq!(budget, base_budget);
        assert!(loop_state.notes.is_empty());
    }
}
