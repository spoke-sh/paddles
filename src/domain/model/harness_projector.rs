use super::harness::{
    GovernorState, HarnessChamber, HarnessSnapshot, HarnessStatus, TimeoutPhase, TimeoutState,
};
use super::turns::TurnEvent;

pub fn derive_harness_snapshot(event: &TurnEvent) -> Option<HarnessSnapshot> {
    match event {
        TurnEvent::HarnessState { .. } => None,
        TurnEvent::IntentClassified { intent } => Some(
            HarnessSnapshot::active(HarnessChamber::Routing)
                .with_detail(format!("intent={}", intent.label())),
        ),
        TurnEvent::InterpretationContext { context } => Some(
            HarnessSnapshot::active(HarnessChamber::Interpretation)
                .with_detail(context.summary.clone()),
        ),
        TurnEvent::GuidanceGraphExpanded {
            depth,
            document_count,
            ..
        } => Some(
            HarnessSnapshot::active(HarnessChamber::Interpretation).with_detail(format!(
                "guidance graph depth {depth}, {document_count} docs"
            )),
        ),
        TurnEvent::RouteSelected { summary } => {
            Some(HarnessSnapshot::active(HarnessChamber::Routing).with_detail(summary.clone()))
        }
        TurnEvent::PlannerCapability {
            provider,
            capability,
        } => Some(
            HarnessSnapshot::active(HarnessChamber::Planning)
                .with_detail(format!("{provider}: {capability}")),
        ),
        TurnEvent::GathererCapability {
            provider,
            capability,
        } => Some(
            HarnessSnapshot::active(HarnessChamber::Gathering)
                .with_detail(format!("{provider}: {capability}")),
        ),
        TurnEvent::PlannerActionSelected {
            sequence, action, ..
        } => Some(
            HarnessSnapshot::active(HarnessChamber::Planning)
                .with_detail(format!("step {sequence}: {action}")),
        ),
        TurnEvent::ThreadCandidateCaptured {
            candidate_id,
            active_thread,
            ..
        } => Some(
            HarnessSnapshot::active(HarnessChamber::Threading)
                .with_detail(format!("{candidate_id} on {active_thread}")),
        ),
        TurnEvent::ThreadDecisionApplied {
            decision,
            target_thread,
            ..
        } => Some(
            HarnessSnapshot::active(HarnessChamber::Threading)
                .with_detail(format!("{decision} -> {target_thread}")),
        ),
        TurnEvent::ThreadMerged {
            source_thread,
            target_thread,
            mode,
            ..
        } => Some(
            HarnessSnapshot::active(HarnessChamber::Threading)
                .with_detail(format!("{source_thread} -> {target_thread} via {mode}")),
        ),
        TurnEvent::PlannerStepProgress {
            step_number,
            step_limit,
            action,
            query,
            ..
        } => {
            let query = query
                .as_deref()
                .map(|value| format!(" — {value}"))
                .unwrap_or_default();
            Some(
                HarnessSnapshot::active(HarnessChamber::Planning)
                    .with_detail(format!("step {step_number}/{step_limit}: {action}{query}")),
            )
        }
        TurnEvent::GathererSearchProgress {
            phase,
            elapsed_seconds,
            eta_seconds,
            strategy,
            detail,
        } => {
            let mut governor =
                GovernorState::active().with_timeout(timeout_state(*elapsed_seconds, *eta_seconds));
            if matches!(
                governor.timeout.phase,
                TimeoutPhase::Stalled | TimeoutPhase::Expired
            ) {
                governor.status = HarnessStatus::Intervening;
                governor.intervention = Some(format!(
                    "search {phase} is {}",
                    governor.timeout.phase.label()
                ));
            }
            let detail = detail
                .clone()
                .or_else(|| {
                    strategy
                        .as_ref()
                        .map(|value| format!("{phase} via {value}"))
                })
                .unwrap_or_else(|| phase.clone());
            Some(
                HarnessSnapshot::active(HarnessChamber::Gathering)
                    .with_governor(governor)
                    .with_detail(detail),
            )
        }
        TurnEvent::GathererSummary {
            provider, summary, ..
        } => Some(
            HarnessSnapshot::active(HarnessChamber::Gathering)
                .with_detail(format!("{provider}: {summary}")),
        ),
        TurnEvent::PlannerSummary {
            strategy,
            mode,
            steps,
            stop_reason,
            ..
        } => {
            let mut detail = format!("{strategy} {mode} ({steps} steps)");
            if let Some(stop_reason) = stop_reason {
                detail.push_str(&format!(" · stop={stop_reason}"));
            }
            Some(HarnessSnapshot::active(HarnessChamber::Planning).with_detail(detail))
        }
        TurnEvent::RefinementApplied { reason, .. } => Some(
            HarnessSnapshot::intervening(HarnessChamber::Governor, reason.clone())
                .with_detail(reason.clone()),
        ),
        TurnEvent::ContextAssembly {
            label,
            hits,
            retained_artifacts,
            ..
        } => Some(
            HarnessSnapshot::active(HarnessChamber::Interpretation).with_detail(format!(
                "{label}: {hits} hits, retained {retained_artifacts}"
            )),
        ),
        TurnEvent::ToolCalled {
            tool_name,
            invocation,
            ..
        } => Some(
            HarnessSnapshot::active(HarnessChamber::Tooling)
                .with_detail(format!("{tool_name}: {invocation}")),
        ),
        TurnEvent::ToolFinished {
            tool_name, summary, ..
        } => Some(
            HarnessSnapshot::active(HarnessChamber::Tooling)
                .with_detail(format!("{tool_name}: {summary}")),
        ),
        TurnEvent::Fallback { stage, reason } => Some(
            HarnessSnapshot::intervening(HarnessChamber::Governor, format!("{stage}: {reason}"))
                .with_detail(format!("{stage}: {reason}")),
        ),
        TurnEvent::ContextStrain { strain } => Some(
            HarnessSnapshot::intervening(
                HarnessChamber::Governor,
                format!("context strain {}", strain.level.label()),
            )
            .with_detail(format!(
                "{} truncation(s), factors: {}",
                strain.truncation_count,
                strain
                    .factors
                    .iter()
                    .map(|factor| factor.label())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        ),
        TurnEvent::SynthesisReady {
            grounded,
            citations,
            insufficient_evidence,
        } => {
            let detail = if *insufficient_evidence {
                "insufficient evidence".to_string()
            } else if *grounded {
                format!("grounded answer with {} citation(s)", citations.len())
            } else {
                "direct answer ready".to_string()
            };
            Some(HarnessSnapshot::active(HarnessChamber::Rendering).with_detail(detail))
        }
    }
}

fn timeout_state(elapsed_seconds: u64, eta_seconds: Option<u64>) -> TimeoutState {
    let deadline_seconds = eta_seconds.map(|eta| elapsed_seconds.saturating_add(eta));
    let phase = if elapsed_seconds >= 120 {
        TimeoutPhase::Expired
    } else if elapsed_seconds >= 45 {
        TimeoutPhase::Stalled
    } else if elapsed_seconds >= 10 {
        TimeoutPhase::Slow
    } else {
        TimeoutPhase::Nominal
    };
    TimeoutState {
        phase,
        elapsed_seconds: Some(elapsed_seconds),
        deadline_seconds,
    }
}

#[cfg(test)]
mod tests {
    use super::derive_harness_snapshot;
    use crate::domain::model::{
        HarnessChamber, HarnessStatus, TimeoutPhase, TurnEvent, TurnIntent,
    };

    #[test]
    fn projector_maps_intent_classification_into_routing_snapshot() {
        let snapshot = derive_harness_snapshot(&TurnEvent::IntentClassified {
            intent: TurnIntent::Planned,
        })
        .expect("snapshot");

        assert_eq!(snapshot.chamber, HarnessChamber::Routing);
        assert_eq!(snapshot.governor.status, HarnessStatus::Active);
        assert_eq!(snapshot.governor.timeout.phase, TimeoutPhase::Nominal);
        assert_eq!(snapshot.detail.as_deref(), Some("intent=planned"));
    }
}
