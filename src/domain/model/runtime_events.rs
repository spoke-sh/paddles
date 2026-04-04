use super::TurnEvent;
use serde::Serialize;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RuntimeEventPresentation {
    pub badge: String,
    pub badge_class: String,
    pub title: String,
    pub detail: String,
    pub text: String,
}

pub fn project_runtime_event(event: &TurnEvent) -> RuntimeEventPresentation {
    match event {
        TurnEvent::IntentClassified { intent } => RuntimeEventPresentation {
            badge: "route".to_string(),
            badge_class: "route".to_string(),
            title: "• Classified".to_string(),
            detail: intent.label().to_string(),
            text: format!("Intent: {}", intent.label()),
        },
        TurnEvent::InterpretationContext { context } => RuntimeEventPresentation {
            badge: "context".to_string(),
            badge_class: "route".to_string(),
            title: format!(
                "• Assembled interpretation context [{} docs, {} hints, {} procedures]",
                context.documents.len(),
                context.tool_hints.len(),
                context.decision_framework.procedures.len()
            ),
            detail: context.summary.clone(),
            text: context.summary.clone(),
        },
        TurnEvent::GuidanceGraphExpanded {
            depth,
            document_count,
            sources,
        } => {
            let detail = format!(
                "depth {depth}: found {document_count} docs ({})",
                sources.join(", ")
            );
            RuntimeEventPresentation {
                badge: "context".to_string(),
                badge_class: "route".to_string(),
                title: "• Expanded guidance graph".to_string(),
                detail: detail.clone(),
                text: detail,
            }
        }
        TurnEvent::RouteSelected { summary } => RuntimeEventPresentation {
            badge: "route".to_string(),
            badge_class: "route".to_string(),
            title: "• Routed".to_string(),
            detail: summary.clone(),
            text: summary.clone(),
        },
        TurnEvent::PlannerCapability {
            provider,
            capability,
        } => {
            let detail = format!("{provider}: {capability}");
            RuntimeEventPresentation {
                badge: "cap".to_string(),
                badge_class: "route".to_string(),
                title: "• Checked planner capability".to_string(),
                detail: detail.clone(),
                text: detail,
            }
        }
        TurnEvent::GathererCapability {
            provider,
            capability,
        } => {
            let detail = format!("{provider}: {capability}");
            RuntimeEventPresentation {
                badge: "cap".to_string(),
                badge_class: "route".to_string(),
                title: "• Checked gatherer capability".to_string(),
                detail: detail.clone(),
                text: detail,
            }
        }
        TurnEvent::PlannerActionSelected {
            sequence,
            action,
            rationale,
        } => RuntimeEventPresentation {
            badge: "planner".to_string(),
            badge_class: "planner".to_string(),
            title: format!("• Planner step {sequence}: {action}"),
            detail: format!("Rationale: {rationale}"),
            text: format!("Step {sequence}: {action}"),
        },
        TurnEvent::ThreadCandidateCaptured {
            candidate_id,
            active_thread,
            prompt,
        } => RuntimeEventPresentation {
            badge: "thread".to_string(),
            badge_class: "planner".to_string(),
            title: "• Captured steering prompt".to_string(),
            detail: format!("{candidate_id} on {active_thread}\n{prompt}"),
            text: format!("{candidate_id} on {active_thread}"),
        },
        TurnEvent::ThreadDecisionApplied {
            candidate_id,
            decision,
            target_thread,
            rationale,
        } => RuntimeEventPresentation {
            badge: "thread".to_string(),
            badge_class: "planner".to_string(),
            title: "• Applied thread decision".to_string(),
            detail: format!(
                "{candidate_id}: {decision} -> {target_thread}\nRationale: {rationale}"
            ),
            text: format!("{candidate_id}: {decision} -> {target_thread}"),
        },
        TurnEvent::ThreadMerged {
            source_thread,
            target_thread,
            mode,
            summary,
        } => RuntimeEventPresentation {
            badge: "merge".to_string(),
            badge_class: "planner".to_string(),
            title: "• Merged thread".to_string(),
            detail: format!(
                "{source_thread} -> {target_thread} via {mode}\n{}",
                summary.as_deref().unwrap_or("No merge summary recorded.")
            ),
            text: format!("{source_thread} -> {target_thread}"),
        },
        TurnEvent::PlannerStepProgress {
            step_number,
            step_limit,
            action,
            query,
            evidence_count,
        } => {
            let title = if let Some(query) = query {
                format!("• Step {step_number}/{step_limit}: {action} — {query}")
            } else {
                format!("• Step {step_number}/{step_limit}: {action}")
            };
            RuntimeEventPresentation {
                badge: "planner".to_string(),
                badge_class: "planner".to_string(),
                title: title.clone(),
                detail: format!("{evidence_count} evidence items"),
                text: title.trim_start_matches("• ").to_string(),
            }
        }
        TurnEvent::GathererSearchProgress {
            phase,
            elapsed_seconds,
            eta_seconds,
            strategy,
            detail,
        } => {
            let elapsed = format_duration_compact(Duration::from_secs(*elapsed_seconds));
            let eta = format_duration_compact(Duration::from_secs(eta_seconds.unwrap_or(0)));
            let strategy_suffix = strategy
                .as_deref()
                .map(|value| format!(" strategy={value}"))
                .unwrap_or_default();
            let detail = detail.clone().unwrap_or_default();
            let fallback = format!("hunting ({phase})");
            let mut text_parts = Vec::new();
            if let Some(strategy) = strategy.as_deref().filter(|value| !value.is_empty()) {
                text_parts.push(strategy.to_string());
            }
            if !detail.is_empty() {
                text_parts.push(detail.clone());
            } else {
                text_parts.push(fallback);
            }
            if eta_seconds.is_some() {
                text_parts.push(format!("eta {eta}"));
            }
            RuntimeEventPresentation {
                badge: "gather".to_string(),
                badge_class: "gatherer".to_string(),
                title: format!("• Hunting ({phase}) — {elapsed} (eta {eta}){strategy_suffix}"),
                detail,
                text: text_parts.join(" · "),
            }
        }
        TurnEvent::GathererSummary {
            provider,
            summary,
            sources,
        } => {
            let detail = if sources.is_empty() {
                summary.clone()
            } else {
                format!("{summary}\nSources: {}", sources.join(", "))
            };
            RuntimeEventPresentation {
                badge: "gather".to_string(),
                badge_class: "gatherer".to_string(),
                title: format!("• Gathered context with {provider}"),
                detail,
                text: summary.clone(),
            }
        }
        TurnEvent::HarnessState { snapshot } => {
            let detail = snapshot.governor_summary(true);
            let mut text_parts = vec![
                snapshot.chamber.to_string(),
                format!("status {}", snapshot.governor.status),
                format!("timeout {}", snapshot.governor.timeout.phase),
            ];
            if let Some(detail) = snapshot.detail.as_deref().filter(|value| !value.is_empty()) {
                text_parts.push(detail.to_string());
            }
            RuntimeEventPresentation {
                badge: "gov".to_string(),
                badge_class: "governor".to_string(),
                title: format!("• Governor: {}", snapshot.chamber),
                detail,
                text: text_parts.join(" · "),
            }
        }
        TurnEvent::PlannerSummary {
            strategy,
            mode,
            turns,
            steps,
            stop_reason,
            ..
        } => {
            let detail = format!(
                "strategy={strategy}, mode={mode}, turns={turns}, steps={steps}, stop={}",
                stop_reason.as_deref().unwrap_or("none"),
            );
            RuntimeEventPresentation {
                badge: "planner".to_string(),
                badge_class: "planner".to_string(),
                title: "• Reviewed planner trace".to_string(),
                detail: detail.clone(),
                text: detail,
            }
        }
        TurnEvent::RefinementApplied {
            reason,
            before_summary,
            after_summary,
        } => RuntimeEventPresentation {
            badge: "refine".to_string(),
            badge_class: "planner".to_string(),
            title: "• Applied interpretation refinement".to_string(),
            detail: format!("{reason}\nbefore: {before_summary}\nafter: {after_summary}"),
            text: reason.clone(),
        },
        TurnEvent::ContextAssembly {
            label,
            hits,
            retained_artifacts,
            pruned_artifacts,
        } => {
            let detail =
                format!("{hits} hit(s), retained {retained_artifacts}, pruned {pruned_artifacts}");
            RuntimeEventPresentation {
                badge: "context".to_string(),
                badge_class: "route".to_string(),
                title: format!("• Assembled workspace context ({label})"),
                detail: detail.clone(),
                text: detail,
            }
        }
        TurnEvent::ToolCalled {
            tool_name,
            invocation,
            ..
        } => RuntimeEventPresentation {
            badge: "tool".to_string(),
            badge_class: "tool".to_string(),
            title: if tool_name == "shell" {
                "• Ran shell".to_string()
            } else {
                format!("• Ran {tool_name}")
            },
            detail: invocation.clone(),
            text: format!("{tool_name}: {invocation}"),
        },
        TurnEvent::ToolFinished {
            tool_name, summary, ..
        } => RuntimeEventPresentation {
            badge: "tool".to_string(),
            badge_class: "tool".to_string(),
            title: format!("• Completed {tool_name}"),
            detail: summary.clone(),
            text: format!("{tool_name} done"),
        },
        TurnEvent::Fallback { stage, reason } => {
            let detail = format!("{stage}: {reason}");
            RuntimeEventPresentation {
                badge: "fallback".to_string(),
                badge_class: "fallback".to_string(),
                title: "• Fell back".to_string(),
                detail: detail.clone(),
                text: detail,
            }
        }
        TurnEvent::ContextStrain { strain } => {
            let factors: Vec<_> = strain.factors.iter().map(|factor| factor.label()).collect();
            let detail = format!(
                "{} truncation(s), factors: [{}]",
                strain.truncation_count,
                factors.join(", ")
            );
            RuntimeEventPresentation {
                badge: "strain".to_string(),
                badge_class: "fallback".to_string(),
                title: format!("• Context strain: {}", strain.level.label()),
                detail: detail.clone(),
                text: detail,
            }
        }
        TurnEvent::SynthesisReady {
            grounded,
            citations,
            insufficient_evidence,
        } => {
            let (title, detail, badge_class, text) = if *insufficient_evidence {
                (
                    "• Reported insufficient evidence".to_string(),
                    "No cited repository sources were available.".to_string(),
                    "fallback".to_string(),
                    "Insufficient evidence".to_string(),
                )
            } else if *grounded {
                (
                    "• Synthesized grounded answer".to_string(),
                    if citations.is_empty() {
                        "Sources: none".to_string()
                    } else {
                        format!("Sources: {}", citations.join(", "))
                    },
                    "synthesis".to_string(),
                    "Grounded".to_string(),
                )
            } else {
                (
                    "• Synthesized direct answer".to_string(),
                    "No repository citations required for this turn.".to_string(),
                    "synthesis".to_string(),
                    "Direct answer".to_string(),
                )
            };
            RuntimeEventPresentation {
                badge: "synth".to_string(),
                badge_class,
                title,
                detail,
                text,
            }
        }
    }
}

fn format_duration_compact(duration: Duration) -> String {
    if duration < Duration::from_secs(1) {
        return format!("{}ms", duration.as_millis());
    }

    if duration < Duration::from_secs(60) {
        return format!("{:.1}s", duration.as_secs_f64());
    }

    if duration < Duration::from_secs(3600) {
        let total_seconds = duration.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        return format!("{minutes}m {seconds:02}s");
    }

    let total_minutes = duration.as_secs() / 60;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{hours}h {minutes:02}m")
}

#[cfg(test)]
mod tests {
    use super::{RuntimeEventPresentation, project_runtime_event};
    use crate::domain::model::{
        GovernorState, HarnessChamber, HarnessSnapshot, HarnessStatus, TimeoutPhase, TimeoutState,
        TurnEvent,
    };

    #[test]
    fn projects_tool_calls_into_runtime_event_presentation() {
        let presentation = project_runtime_event(&TurnEvent::ToolCalled {
            call_id: "tool-1".to_string(),
            tool_name: "shell".to_string(),
            invocation: "git status --short".to_string(),
        });

        assert_eq!(
            presentation,
            RuntimeEventPresentation {
                badge: "tool".to_string(),
                badge_class: "tool".to_string(),
                title: "• Ran shell".to_string(),
                detail: "git status --short".to_string(),
                text: "shell: git status --short".to_string(),
            }
        );
    }

    #[test]
    fn projects_hunting_progress_into_runtime_event_presentation() {
        let presentation = project_runtime_event(&TurnEvent::GathererSearchProgress {
            phase: "Indexing".to_string(),
            elapsed_seconds: 110,
            eta_seconds: Some(0),
            strategy: Some("bm25".to_string()),
            detail: Some("indexing 75914/75934 files".to_string()),
        });

        assert_eq!(presentation.badge, "gather");
        assert_eq!(presentation.badge_class, "gatherer");
        assert_eq!(
            presentation.title,
            "• Hunting (Indexing) — 1m 50s (eta 0ms) strategy=bm25"
        );
        assert_eq!(presentation.detail, "indexing 75914/75934 files");
        assert_eq!(
            presentation.text,
            "bm25 · indexing 75914/75934 files · eta 0ms"
        );
    }

    #[test]
    fn projects_governor_state_into_runtime_event_presentation() {
        let presentation = project_runtime_event(&TurnEvent::HarnessState {
            snapshot: HarnessSnapshot {
                chamber: HarnessChamber::Gathering,
                governor: GovernorState {
                    status: HarnessStatus::Active,
                    timeout: TimeoutState {
                        phase: TimeoutPhase::Slow,
                        elapsed_seconds: Some(9),
                        deadline_seconds: Some(30),
                    },
                    intervention: None,
                },
                detail: Some("indexing 4/10 files".to_string()),
            },
        });

        assert_eq!(presentation.badge, "gov");
        assert_eq!(presentation.title, "• Governor: gathering");
        assert!(presentation.detail.contains("status=active"));
        assert!(presentation.detail.contains("timeout=slow"));
        assert!(presentation.text.contains("gathering"));
    }
}
