use super::{AppliedEdit, TurnEvent};
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
                format!("watch {}", snapshot.governor.timeout.phase.watch_label()),
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
        TurnEvent::ToolOutput {
            tool_name,
            stream,
            output,
            ..
        } => RuntimeEventPresentation {
            badge: "term".to_string(),
            badge_class: "tool-terminal".to_string(),
            title: format!("• {tool_name} {stream}"),
            detail: output.clone(),
            text: format!("{tool_name} {stream}"),
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
        TurnEvent::WorkspaceEditApplied {
            tool_name, edit, ..
        } => RuntimeEventPresentation {
            badge: "tool".to_string(),
            badge_class: "tool-diff".to_string(),
            title: format!("• Applied {tool_name}"),
            detail: format_applied_edit_detail(edit),
            text: format_applied_edit_text(tool_name, edit),
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

pub fn project_runtime_event_for_tui(event: &TurnEvent, verbose: u8) -> RuntimeEventPresentation {
    let mut presentation = project_runtime_event(event);

    match event {
        TurnEvent::InterpretationContext { context } => {
            let summary = if context.summary.trim().is_empty() {
                "(no details)".to_string()
            } else {
                context.summary.clone()
            };
            let content = if verbose >= 2 {
                if context.summary.trim().is_empty() && context.sources().is_empty() {
                    "No operator interpretation context was assembled.".to_string()
                } else {
                    context.render()
                }
            } else if verbose == 1 {
                let mut sections = vec![summary];
                let sources = context.sources();
                if !sources.is_empty() {
                    sections.push(format!("Sources: {}", sources.join(", ")));
                }
                for doc in &context.documents {
                    sections.push(format!(
                        "--- {} [{:?}] ---\n{}",
                        doc.source, doc.category, doc.excerpt
                    ));
                }
                if !context.tool_hints.is_empty() {
                    sections.push("--- Tool Hints ---".to_string());
                    sections.extend(context.tool_hints.iter().map(|hint| {
                        format!(
                            "- {} ({}) — {}",
                            hint.action.summary(),
                            hint.source,
                            hint.note
                        )
                    }));
                }
                sections.join("\n\n")
            } else {
                let mut content = summary;
                let sources = context.sources();
                if !sources.is_empty() {
                    if !content.is_empty() {
                        content.push('\n');
                    }
                    content.push_str("Sources: ");
                    content.push_str(&sources.join(", "));
                }
                content
            };
            presentation.detail = content.clone();
            presentation.text = content;
        }
        TurnEvent::PlannerStepProgress { .. } if verbose == 0 => {
            presentation.detail = String::new();
            presentation.text = String::new();
        }
        TurnEvent::PlannerSummary {
            strategy,
            mode,
            turns,
            steps,
            stop_reason,
            active_branch_id,
            branch_count,
            frontier_count,
            node_count,
            edge_count,
            retained_artifact_count,
        } if verbose >= 2 => {
            let opt = |value: Option<usize>| {
                value
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "n/a".to_string())
            };
            let text = format!(
                "strategy={strategy}, mode={mode}, turns={turns}, steps={steps}, stop={}\nGraph: nodes={}, edges={}, branches={}, frontier={}, active={}, retained={}",
                stop_reason.as_deref().unwrap_or("none"),
                opt(*node_count),
                opt(*edge_count),
                opt(*branch_count),
                opt(*frontier_count),
                active_branch_id.as_deref().unwrap_or("none"),
                opt(*retained_artifact_count),
            );
            presentation.detail = text.clone();
            presentation.text = text;
        }
        _ => {}
    }

    presentation
}

fn format_applied_edit_detail(edit: &AppliedEdit) -> String {
    let files = if edit.files.is_empty() {
        "(unknown file)".to_string()
    } else {
        edit.files.join(", ")
    };
    if edit.diff.trim().is_empty() {
        format!(
            "Files: {files}\nChange: +{} -{}",
            edit.insertions, edit.deletions
        )
    } else {
        format!(
            "Files: {files}\nChange: +{} -{}\n\n{}",
            edit.insertions, edit.deletions, edit.diff
        )
    }
}

fn format_applied_edit_text(tool_name: &str, edit: &AppliedEdit) -> String {
    let files = if edit.files.is_empty() {
        "unknown file".to_string()
    } else {
        edit.files.join(", ")
    };
    format!(
        "{tool_name}: {files} (+{} -{})",
        edit.insertions, edit.deletions
    )
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
    use super::{RuntimeEventPresentation, project_runtime_event, project_runtime_event_for_tui};
    use crate::domain::model::{
        AppliedEdit, GovernorState, HarnessChamber, HarnessSnapshot, HarnessStatus, TimeoutPhase,
        TimeoutState, TurnEvent,
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
    fn projects_terminal_output_into_runtime_event_presentation() {
        let presentation = project_runtime_event(&TurnEvent::ToolOutput {
            call_id: "tool-1".to_string(),
            tool_name: "shell".to_string(),
            stream: "stdout".to_string(),
            output: "alpha\nbeta".to_string(),
        });

        assert_eq!(
            presentation,
            RuntimeEventPresentation {
                badge: "term".to_string(),
                badge_class: "tool-terminal".to_string(),
                title: "• shell stdout".to_string(),
                detail: "alpha\nbeta".to_string(),
                text: "shell stdout".to_string(),
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
    fn projects_applied_workspace_edits_into_diff_presentations() {
        let presentation = project_runtime_event(&TurnEvent::WorkspaceEditApplied {
            call_id: "tool-2".to_string(),
            tool_name: "apply_patch".to_string(),
            edit: AppliedEdit {
                files: vec!["src/app.rs".to_string()],
                diff: "--- a/src/app.rs\n+++ b/src/app.rs\n@@ -1 +1 @@\n-old\n+new".to_string(),
                insertions: 1,
                deletions: 1,
            },
        });

        assert_eq!(presentation.badge, "tool");
        assert_eq!(presentation.badge_class, "tool-diff");
        assert_eq!(presentation.title, "• Applied apply_patch");
        assert!(presentation.detail.contains("Files: src/app.rs"));
        assert!(presentation.detail.contains("+new"));
        assert_eq!(presentation.text, "apply_patch: src/app.rs (+1 -1)");
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
        assert!(presentation.detail.contains("watch=slow"));
        assert!(presentation.detail.contains("projected_total=30s"));
        assert!(!presentation.detail.contains("timeout="));
        assert!(presentation.text.contains("gathering"));
        assert!(presentation.text.contains("watch slow"));
    }

    #[test]
    fn projects_planner_summary_adds_graph_stats_when_tui_verbosity_is_high() {
        let presentation = project_runtime_event_for_tui(
            &TurnEvent::PlannerSummary {
                strategy: "direct".to_string(),
                mode: "single".to_string(),
                turns: 1,
                steps: 3,
                stop_reason: Some("planner_budget".to_string()),
                active_branch_id: None,
                branch_count: None,
                frontier_count: None,
                node_count: Some(12),
                edge_count: Some(4),
                retained_artifact_count: Some(0),
            },
            2,
        );

        assert_eq!(
            presentation.text,
            "strategy=direct, mode=single, turns=1, steps=3, stop=planner_budget\nGraph: nodes=12, edges=4, branches=n/a, frontier=n/a, active=none, retained=0"
        );
        assert_eq!(presentation.title, "• Reviewed planner trace");
    }

    #[test]
    fn projects_planner_step_progress_hides_details_at_tui_low_verbosity() {
        let presentation = project_runtime_event_for_tui(
            &TurnEvent::PlannerStepProgress {
                step_number: 2,
                step_limit: 4,
                action: "search".to_string(),
                query: None,
                evidence_count: 7,
            },
            0,
        );

        assert_eq!(presentation.detail, "");
        assert_eq!(presentation.title, "• Step 2/4: search");
    }
}
