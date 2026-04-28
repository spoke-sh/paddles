use super::*;

pub(super) struct RecursiveControlChamber<'a> {
    service: &'a AgentRuntime,
}

impl<'a> RecursiveControlChamber<'a> {
    pub(super) const fn new(service: &'a AgentRuntime) -> Self {
        Self { service }
    }

    pub(super) async fn execute_recursive_planner_loop(
        &self,
        prompt: &str,
        context: PlannerLoopContext,
        initial_decision: Option<RecursivePlannerDecision>,
        trace: Arc<StructuredTurnTrace>,
    ) -> Result<PlannerLoopOutcome> {
        let mut context = context;
        let base_budget =
            planner_budget_for_turn(context.instruction_frame.as_ref(), &context.initial_edit);
        let planner_loop_service = self.service.planner_loop_service();
        let execution_contract_service = self.service.execution_contract_service();
        let mut budget = planner_loop_service.budget_for_replan_attempt(&base_budget, 0);
        let harness_profile = context.prepared.harness_profile();
        let mut loop_state = PlannerLoopState {
            target_resolution: context.initial_edit.resolution.clone(),
            refinement_policy: harness_profile.active_refinement_policy(),
            ..PlannerLoopState::default()
        };
        let mut used_workspace_resources = false;
        let mut stop_reason = None;
        let mut direct_answer = None;
        let mut instruction_frame = context.instruction_frame.clone();
        let mut pending_initial_decision = initial_decision;
        let mut pending_deliberation_signals = DeliberationSignals::default();
        let mut steps_without_new_evidence = 0usize;
        let mut replan_count = 0usize;
        let mut sequence = 1usize;
        let gatherer_provider = context
            .prepared
            .gatherer
            .as_ref()
            .map(|lane| lane.label.clone())
            .unwrap_or_else(|| "workspace".to_string());

        loop {
            if let Some(control_outcome) = self
                .service
                .apply_turn_controls_at_safe_checkpoint(&context, &trace)
                .await?
            {
                return Ok(control_outcome);
            }

            if sequence > budget.max_steps {
                if let Some(replan_event) = planner_loop_service.activate_replan(
                    "planner-budget-exhausted",
                    PlannerLoopReplanActivation {
                        instruction_frame: instruction_frame.as_ref(),
                        base_budget: &base_budget,
                        completed_replans: &mut replan_count,
                        budget: &mut budget,
                        loop_state: &mut loop_state,
                    },
                ) {
                    trace.emit(TurnEvent::Fallback {
                        stage: replan_event.stage.to_string(),
                        reason: replan_event.reason,
                    });
                    continue;
                }
                break;
            }

            let evidence_count_before = loop_state.evidence_items.len();
            let planner_selected_this_step = pending_initial_decision.is_none();
            sync_deliberation_signal_note(&mut loop_state, &pending_deliberation_signals);
            let mut decision = if let Some(decision) = pending_initial_decision.take() {
                decision
            } else {
                context.workspace_capability_surface = self
                    .service
                    .workspace_action_executor()
                    .capability_surface();
                let request = PlannerRequest::new(
                    prompt,
                    self.service.workspace_root.clone(),
                    context.interpretation.clone(),
                    budget.clone(),
                )
                .with_collaboration(context.collaboration.clone())
                .with_recent_turns(context.recent_turns.clone())
                .with_recent_thread_summary(context.recent_thread_summary.clone())
                .with_runtime_notes(planner_runtime_notes(
                    context.gatherer.as_ref(),
                    &context.specialist_runtime_notes,
                    &context.collaboration,
                ))
                .with_execution_contract(execution_contract_service.build(
                    ExecutionContractContext {
                        workspace_capability_surface: &context.workspace_capability_surface,
                        execution_hands: &context.execution_hands,
                        governance_profile: context.governance_profile.as_ref(),
                        external_capabilities: &context.external_capabilities,
                        gatherer: context.gatherer.as_deref(),
                        collaboration: &context.collaboration,
                        instruction_frame: instruction_frame.as_ref(),
                        grounding: context.grounding.as_ref(),
                    },
                ))
                .with_loop_state(loop_state.clone())
                .with_resolver(context.resolver.clone())
                .with_entity_resolver(context.entity_resolver.clone());
                context
                    .planner_engine
                    .select_next_action(&request, trace.clone() as Arc<dyn TurnEventSink>)
                    .await?
            };

            if planner_selected_this_step {
                decision = review_decision_under_signals(
                    prompt,
                    &context,
                    &budget,
                    &loop_state,
                    decision,
                    DecisionReviewFrame {
                        deliberation_signals: &pending_deliberation_signals,
                        workspace_root: &self.service.workspace_root,
                        trace: trace.clone(),
                    },
                )
                .await?;
            }

            decision = sanitize_recursive_planner_decision_for_collaboration(
                &context.collaboration,
                decision,
            );
            instruction_frame =
                merge_instruction_frame_with_edit_signal(instruction_frame, &decision.edit);
            if let Some(resolution) = decision.edit.resolution.clone() {
                loop_state.target_resolution = Some(resolution);
            }
            if planner_selected_this_step {
                let (controller_summary, signal_summary) = compile_recursive_paddles_rationale(
                    &decision.action,
                    &loop_state.evidence_items,
                    &pending_deliberation_signals,
                );
                trace.emit(TurnEvent::PlannerActionSelected {
                    sequence,
                    action: decision.action.summary(),
                    rationale: decision.rationale.clone(),
                    signal_summary: signal_summary.clone(),
                    controller_summary: Some(controller_summary.clone()),
                });
                trace.record_planner_action(
                    &decision.action.summary(),
                    &decision.rationale,
                    signal_summary.as_deref(),
                    Some(controller_summary.as_str()),
                    None,
                );
            }

            trace.emit(TurnEvent::PlannerStepProgress {
                step_number: sequence,
                step_limit: budget.max_steps,
                action: decision.action.summary(),
                query: decision.action.target_query(),
                evidence_count: loop_state.evidence_items.len(),
            });

            let mut accepted_stop = false;
            let outcome = if let Some(outcome) = collaboration_boundary_for_action(
                &context.collaboration,
                &decision.action,
                &decision.edit,
            ) {
                trace.emit(TurnEvent::Fallback {
                    stage: "collaboration-mode".to_string(),
                    reason: outcome.summary.clone(),
                });
                if let Some(clarification) = outcome.clarification.clone() {
                    trace.emit(TurnEvent::StructuredClarificationChanged {
                        result: clarification,
                    });
                }
                direct_answer = Some(outcome.response);
                stop_reason = Some(outcome.reason);
                accepted_stop = true;
                outcome.summary
            } else {
                match &decision.action {
                    PlannerAction::Workspace { action } => match action {
                        WorkspaceAction::Search {
                            query,
                            mode,
                            strategy,
                            retrievers,
                            intent,
                        } => {
                            if search_steps(&loop_state) >= budget.max_searches {
                                stop_reason = Some("search-budget-exhausted".to_string());
                                "planner search budget exhausted".to_string()
                            } else {
                                self.service.execute_planner_gather_step(
                                &context,
                                &mut loop_state,
                                trace.clone(),
                                &gatherer_provider,
                                PlannerGatherSpec {
                                    query: query.clone(),
                                    intent_reason: intent
                                        .clone()
                                        .unwrap_or_else(|| "planner-search".to_string()),
                                    mode: *mode,
                                    strategy: *strategy,
                                    retrievers: retrievers.clone(),
                                    max_evidence_items: budget.max_evidence_items,
                                    success_summary_override: None,
                                    no_bundle_message: "planner search returned no evidence bundle",
                                    failure_label: "planner search failed",
                                    unavailable_label: "planner search backend unavailable",
                                    missing_backend_message:
                                        "no gatherer backend is configured for planner search",
                                },
                                &mut used_workspace_resources,
                            )
                            .await
                            }
                        }
                        WorkspaceAction::Inspect { command } => {
                            if inspect_steps(&loop_state) >= budget.max_inspects {
                                stop_reason = Some("inspect-budget-exhausted".to_string());
                                "planner inspect budget exhausted".to_string()
                            } else {
                                let call_id = format!("planner-tool-{sequence}");
                                trace.emit(TurnEvent::ToolCalled {
                                    call_id: call_id.clone(),
                                    tool_name: "inspect".to_string(),
                                    invocation: command.clone(),
                                });
                                match planner_action_execution::run_planner_inspect_command(
                                    &self.service.workspace_root,
                                    self.service.execution_hand_registry(),
                                    command,
                                    &call_id,
                                    trace.as_ref(),
                                ) {
                                    Ok(output) => {
                                        emit_execution_governance_decision(
                                            trace.as_ref(),
                                            Some(&call_id),
                                            Some("inspect"),
                                            output.governance_request,
                                            output.governance_outcome,
                                        );
                                        if !output.command_succeeded {
                                            let summary =
                                                format!("inspect failed: {}", output.summary);
                                            trace.emit(TurnEvent::ToolFinished {
                                                call_id,
                                                tool_name: "inspect".to_string(),
                                                summary: summary.clone(),
                                            });
                                            append_evidence_item(
                                                &mut loop_state.evidence_items,
                                                EvidenceItem {
                                                    source: format!("command: {command}"),
                                                    snippet: trim_for_planner(&summary, 800),
                                                    rationale: decision.rationale.clone(),
                                                    rank: 0,
                                                },
                                                budget.max_evidence_items,
                                            );
                                            used_workspace_resources = true;
                                            summary
                                        } else {
                                            let summary =
                                                planner_action_execution::planner_terminal_tool_success_summary(
                                                "inspect",
                                                &output.summary,
                                            );
                                            trace.emit(TurnEvent::ToolFinished {
                                                call_id,
                                                tool_name: "inspect".to_string(),
                                                summary,
                                            });
                                            append_evidence_item(
                                                &mut loop_state.evidence_items,
                                                EvidenceItem {
                                                    source: format!("command: {command}"),
                                                    snippet: trim_for_planner(&output.summary, 800),
                                                    rationale: decision.rationale.clone(),
                                                    rank: 0,
                                                },
                                                budget.max_evidence_items,
                                            );
                                            used_workspace_resources = true;
                                            format!("inspected {command}")
                                        }
                                    }
                                    Err(err) => {
                                        let summary = format!("inspect failed: {err:#}");
                                        trace.emit(TurnEvent::ToolFinished {
                                            call_id,
                                            tool_name: "inspect".to_string(),
                                            summary: summary.clone(),
                                        });
                                        append_evidence_item(
                                            &mut loop_state.evidence_items,
                                            EvidenceItem {
                                                source: format!("command: {command}"),
                                                snippet: trim_for_planner(&summary, 800),
                                                rationale: decision.rationale.clone(),
                                                rank: 0,
                                            },
                                            budget.max_evidence_items,
                                        );
                                        used_workspace_resources = true;
                                        summary
                                    }
                                }
                            }
                        }
                        WorkspaceAction::Shell { command } => {
                            let call_id = format!("planner-tool-{sequence}");
                            trace.emit(TurnEvent::ToolCalled {
                                call_id: call_id.clone(),
                                tool_name: "shell".to_string(),
                                invocation: command.clone(),
                            });
                            match planner_action_execution::run_planner_shell_command(
                                &self.service.workspace_root,
                                self.service.execution_hand_registry(),
                                command,
                                &call_id,
                                trace.as_ref(),
                            ) {
                                Ok(result) => {
                                    emit_execution_governance_decision(
                                        trace.as_ref(),
                                        Some(&call_id),
                                        Some("shell"),
                                        result.governance_request,
                                        result.governance_outcome,
                                    );
                                    if result.command_succeeded {
                                        let summary =
                                            planner_action_execution::planner_terminal_tool_success_summary(
                                            "shell",
                                            &result.summary,
                                        );
                                        trace.emit(TurnEvent::ToolFinished {
                                            call_id,
                                            tool_name: "shell".to_string(),
                                            summary,
                                        });
                                        append_evidence_item(
                                            &mut loop_state.evidence_items,
                                            EvidenceItem {
                                                source: format!("command: {command}"),
                                                snippet: trim_for_planner(&result.summary, 1_200),
                                                rationale: decision.rationale.clone(),
                                                rank: 0,
                                            },
                                            budget.max_evidence_items,
                                        );
                                        if let Some(frame) = instruction_frame.as_mut() {
                                            frame.note_successful_workspace_action(action);
                                        }
                                        used_workspace_resources = true;
                                        result.summary
                                    } else {
                                        let summary =
                                            format!("Tool `shell` failed: {}", result.summary);
                                        trace.emit(TurnEvent::ToolFinished {
                                            call_id,
                                            tool_name: "shell".to_string(),
                                            summary: summary.clone(),
                                        });
                                        append_evidence_item(
                                            &mut loop_state.evidence_items,
                                            EvidenceItem {
                                                source: format!("command: {command}"),
                                                snippet: trim_for_planner(&summary, 1_200),
                                                rationale: decision.rationale.clone(),
                                                rank: 0,
                                            },
                                            budget.max_evidence_items,
                                        );
                                        used_workspace_resources = true;
                                        summary
                                    }
                                }
                                Err(err) => {
                                    let summary = format!("Tool `shell` failed: {err:#}");
                                    trace.emit(TurnEvent::ToolFinished {
                                        call_id,
                                        tool_name: "shell".to_string(),
                                        summary: summary.clone(),
                                    });
                                    append_evidence_item(
                                        &mut loop_state.evidence_items,
                                        EvidenceItem {
                                            source: format!("command: {command}"),
                                            snippet: trim_for_planner(&summary, 1_200),
                                            rationale: decision.rationale.clone(),
                                            rank: 0,
                                        },
                                        budget.max_evidence_items,
                                    );
                                    used_workspace_resources = true;
                                    summary
                                }
                            }
                        }
                        WorkspaceAction::ExternalCapability { invocation } => {
                            let call_id = format!("planner-tool-{sequence}");
                            let broker = self.service.external_capability_broker();
                            let descriptor = broker.descriptor(&invocation.capability_id);
                            trace.emit(TurnEvent::ToolCalled {
                                call_id: call_id.clone(),
                                tool_name: action.label().to_string(),
                                invocation: external_capability_execution::format_external_capability_invocation(
                                    descriptor.as_ref(),
                                    invocation,
                                ),
                            });
                            let summary = external_capability_execution::execute_external_capability_action(
                                broker,
                                context
                                    .prepared
                                    .harness_profile()
                                    .active_execution_governance(),
                                self.service.execution_hand_registry().execution_policy().as_ref(),
                                invocation,
                                external_capability_execution::ExternalCapabilityExecutionFrame {
                                    rationale: decision.rationale.as_str(),
                                    evidence_limit: budget.max_evidence_items,
                                    evidence_items: &mut loop_state.evidence_items,
                                    call_id: &call_id,
                                    event_sink: trace.as_ref(),
                                },
                            );
                            used_workspace_resources = true;
                            summary
                        }
                        WorkspaceAction::Read { .. }
                        | WorkspaceAction::ListFiles { .. }
                        | WorkspaceAction::Diff { .. }
                        | WorkspaceAction::WriteFile { .. }
                        | WorkspaceAction::ReplaceInFile { .. }
                        | WorkspaceAction::ApplyPatch { .. }
                        | WorkspaceAction::SemanticDefinitions { .. }
                        | WorkspaceAction::SemanticReferences { .. }
                        | WorkspaceAction::SemanticSymbols { .. }
                        | WorkspaceAction::SemanticHover { .. }
                        | WorkspaceAction::SemanticDiagnostics { .. } => {
                            let previous_resolution = loop_state.target_resolution.clone();
                            maybe_promote_missing_resolution_for_mutation(
                                &self.service.workspace_root,
                                &context.initial_edit.candidate_files,
                                &mut loop_state,
                                action,
                            );
                            if previous_resolution != loop_state.target_resolution
                                && let Some(outcome @ EntityResolutionOutcome::Resolved { .. }) =
                                    loop_state.target_resolution.as_ref()
                            {
                                trace.record_entity_resolution_outcome(
                                    outcome,
                                    "exact-mutation-path",
                                );
                            }
                            if let Some((reason, summary)) =
                                unresolved_target_mutation_boundary(action, &loop_state)
                            {
                                trace.emit(TurnEvent::Fallback {
                                    stage: "entity-resolution".to_string(),
                                    reason: summary.clone(),
                                });
                                stop_reason = Some(reason);
                                accepted_stop = true;
                                summary
                            } else if matches!(action, WorkspaceAction::Read { .. })
                                && read_steps(&loop_state) >= budget.max_reads
                            {
                                stop_reason = Some("read-budget-exhausted".to_string());
                                "planner read budget exhausted".to_string()
                            } else {
                                let call_id = format!("planner-tool-{sequence}");
                                trace.emit(TurnEvent::ToolCalled {
                                    call_id: call_id.clone(),
                                    tool_name: action.label().to_string(),
                                    invocation: action.describe(),
                                });
                                match self
                                    .service
                                    .workspace_action_executor()
                                    .execute_workspace_action(
                                        action,
                                        WorkspaceActionExecutionFrame {
                                            call_id: &call_id,
                                            event_sink: trace.as_ref(),
                                        },
                                    ) {
                                    Ok(result) => {
                                        if let (
                                            Some(governance_request),
                                            Some(governance_outcome),
                                        ) = (
                                            result.governance_request.clone(),
                                            result.governance_outcome.clone(),
                                        ) {
                                            emit_execution_governance_decision(
                                                trace.as_ref(),
                                                Some(&call_id),
                                                Some(action.label()),
                                                governance_request,
                                                governance_outcome,
                                            );
                                        }
                                        if let Some(frame) = instruction_frame.as_mut() {
                                            frame.note_successful_workspace_action(action);
                                        }
                                        if let Some(edit) = result.applied_edit.clone() {
                                            trace.emit(TurnEvent::WorkspaceEditApplied {
                                                call_id,
                                                tool_name: result.name.to_string(),
                                                edit,
                                            });
                                        } else {
                                            trace.emit(TurnEvent::ToolFinished {
                                                call_id,
                                                tool_name: result.name.to_string(),
                                                summary: result.summary.clone(),
                                            });
                                        }
                                        append_evidence_item(
                                            &mut loop_state.evidence_items,
                                            EvidenceItem {
                                                source: planner_action_execution::workspace_action_evidence_source(action),
                                                snippet: trim_for_planner(&result.summary, 1_200),
                                                rationale: decision.rationale.clone(),
                                                rank: 0,
                                            },
                                            budget.max_evidence_items,
                                        );
                                        used_workspace_resources = true;
                                        result.summary
                                    }
                                    Err(err) => {
                                        let summary =
                                            format!("Tool `{}` failed: {err:#}", action.label());
                                        trace.emit(TurnEvent::ToolFinished {
                                            call_id,
                                            tool_name: action.label().to_string(),
                                            summary: summary.clone(),
                                        });
                                        append_evidence_item(
                                            &mut loop_state.evidence_items,
                                            EvidenceItem {
                                                source: planner_action_execution::workspace_action_evidence_source(action),
                                                snippet: trim_for_planner(&summary, 1_200),
                                                rationale: decision.rationale.clone(),
                                                rank: 0,
                                            },
                                            budget.max_evidence_items,
                                        );
                                        used_workspace_resources = true;
                                        summary
                                    }
                                }
                            }
                        }
                    },
                    PlannerAction::Refine {
                        query,
                        mode,
                        strategy,
                        retrievers,
                        ..
                    } => {
                        if search_steps(&loop_state) >= budget.max_searches {
                            stop_reason = Some("search-budget-exhausted".to_string());
                            "planner refine budget exhausted".to_string()
                        } else {
                            self.service.execute_planner_gather_step(
                            &context,
                            &mut loop_state,
                            trace.clone(),
                            &gatherer_provider,
                            PlannerGatherSpec {
                                query: query.clone(),
                                intent_reason: "planner-refine".to_string(),
                                mode: *mode,
                                strategy: *strategy,
                                retrievers: retrievers.clone(),
                                max_evidence_items: budget.max_evidence_items,
                                success_summary_override: Some(format!(
                                    "refined search toward `{query}`"
                                )),
                                no_bundle_message: "planner refine returned no evidence bundle",
                                failure_label: "planner refine failed",
                                unavailable_label: "planner refine backend unavailable",
                                missing_backend_message:
                                    "no gatherer backend is configured for refined planner search",
                            },
                            &mut used_workspace_resources,
                        )
                        .await
                        }
                    }
                    PlannerAction::Branch { branches, .. } => {
                        for branch in branches.iter().take(budget.max_branch_factor) {
                            let exists = loop_state
                                .pending_branches
                                .iter()
                                .any(|pending| pending.label == *branch);
                            if !exists {
                                let branch_id = trace.session.next_branch_id();
                                let branch_trace = trace.declare_branch(
                                    branch_id,
                                    branch,
                                    Some(decision.rationale.as_str()),
                                    None,
                                );
                                loop_state.pending_branches.push(branch_trace);
                            }
                        }
                        format!(
                            "queued {} planner branch(es)",
                            branches.len().min(budget.max_branch_factor)
                        )
                    }
                    PlannerAction::Stop { reason } => {
                        if let Some(frame) = instruction_frame
                            .as_ref()
                            .filter(|frame| frame.has_pending_workspace_obligation())
                        {
                            let note = instruction_unsatisfied_note(frame);
                            if !loop_state.notes.contains(&note) {
                                loop_state.notes.push(note.clone());
                            }
                            trace.emit(TurnEvent::Fallback {
                                stage: "instruction-manifold".to_string(),
                                reason: note.clone(),
                            });
                            direct_answer = None;
                            stop_reason = Some("instruction-unsatisfied".to_string());
                            "planner stop converted into a blocked reply because the requested applied edit is still unsatisfied"
                            .to_string()
                        } else {
                            direct_answer =
                                stop_reason_direct_answer(reason, decision.answer.clone());
                            stop_reason = Some(reason.clone());
                            accepted_stop = true;
                            format!("planner requested synthesis: {reason}")
                        }
                    }
                }
            };

            loop_state.steps.push(PlannerStepRecord {
                step_id: format!("planner-step-{sequence}"),
                sequence,
                branch_id: None,
                action: decision.action.clone(),
                outcome: outcome.clone(),
            });
            pending_deliberation_signals =
                extract_deliberation_signals(decision.deliberation_state.as_ref());
            sync_deliberation_signal_note(&mut loop_state, &pending_deliberation_signals);

            let evidence_count_after = loop_state.evidence_items.len();
            if evidence_count_after > evidence_count_before {
                steps_without_new_evidence = 0;
            } else {
                steps_without_new_evidence += 1;
            }

            if stop_reason.is_none() && !matches!(decision.action, PlannerAction::Stop { .. }) {
                let refinement_reason = self.service.mid_loop_refinement_reason(
                    sequence,
                    &loop_state,
                    steps_without_new_evidence,
                    &pending_deliberation_signals,
                );
                if let Some(refinement_reason) = refinement_reason
                    && let Some(updated_context) = self
                        .service
                        .derive_mid_loop_interpretation_context(
                            prompt,
                            &context,
                            &loop_state,
                            &evidence_count_before,
                            &trace,
                        )
                        .await
                    && updated_context != context.interpretation
                {
                    let before_summary = context.interpretation.summary.clone();
                    let after_summary = updated_context.summary.clone();
                    loop_state.refinement_count += 1;
                    loop_state.last_refinement_step = Some(sequence);
                    let refinement_signature =
                        AgentRuntime::mid_loop_refinement_signature(&updated_context);
                    if !self.service.mid_loop_refinement_signature_is_stable(
                        &loop_state.refinement_policy,
                        &loop_state.refinement_signatures,
                        &refinement_signature,
                    ) {
                        trace.emit(TurnEvent::Fallback {
                            stage: "refinement-guard".to_string(),
                            reason:
                                "oscillation guard prevented refinement to recently seen interpretation signature"
                                    .to_string(),
                        });
                        sequence += 1;
                        continue;
                    }
                    loop_state
                        .refinement_signatures
                        .push(refinement_signature.clone());
                    if loop_state.refinement_policy.signature_history_limit > 0 {
                        let limit = loop_state.refinement_policy.signature_history_limit;
                        if loop_state.refinement_signatures.len() > limit {
                            let overflow =
                                loop_state.refinement_signatures.len().saturating_sub(limit);
                            loop_state.refinement_signatures.drain(0..overflow);
                        }
                    }
                    context.interpretation = updated_context;
                    trace.emit(TurnEvent::RefinementApplied {
                        reason: refinement_reason,
                        before_summary,
                        after_summary,
                    });
                }
            }

            if accepted_stop {
                break;
            }

            if let Some(reason) = stop_reason.clone() {
                if let Some(replan_event) = planner_loop_service.activate_replan(
                    &reason,
                    PlannerLoopReplanActivation {
                        instruction_frame: instruction_frame.as_ref(),
                        base_budget: &base_budget,
                        completed_replans: &mut replan_count,
                        budget: &mut budget,
                        loop_state: &mut loop_state,
                    },
                ) {
                    trace.emit(TurnEvent::Fallback {
                        stage: replan_event.stage.to_string(),
                        reason: replan_event.reason,
                    });
                    stop_reason = None;
                    sequence += 1;
                    continue;
                }
                break;
            }

            sequence += 1;
        }

        let completed = stop_reason.is_some();
        let stop_reason = annotate_stop_reason_for_pending_instruction(
            stop_reason.unwrap_or_else(|| "planner-budget-exhausted".to_string()),
            instruction_frame.as_ref(),
        );
        trace.emit(TurnEvent::PlannerSummary {
            strategy: "model-driven".to_string(),
            mode: loop_state
                .latest_gatherer_trace
                .as_ref()
                .map(|planner| planner.mode.label().to_string())
                .unwrap_or_else(|| RetrievalMode::Linear.label().to_string()),
            turns: loop_state.steps.len(),
            steps: loop_state.steps.len(),
            stop_reason: Some(stop_reason.clone()),
            active_branch_id: loop_state
                .latest_gatherer_trace
                .as_ref()
                .and_then(|planner| planner.graph_episode.as_ref())
                .and_then(|graph| graph.active_branch_id.clone()),
            branch_count: loop_state
                .latest_gatherer_trace
                .as_ref()
                .and_then(|planner| planner.graph_episode.as_ref())
                .map(|graph| graph.branches.len()),
            frontier_count: loop_state
                .latest_gatherer_trace
                .as_ref()
                .and_then(|planner| planner.graph_episode.as_ref())
                .map(|graph| graph.frontier.len()),
            node_count: loop_state
                .latest_gatherer_trace
                .as_ref()
                .and_then(|planner| planner.graph_episode.as_ref())
                .map(|graph| graph.nodes.len()),
            edge_count: loop_state
                .latest_gatherer_trace
                .as_ref()
                .and_then(|planner| planner.graph_episode.as_ref())
                .map(|graph| graph.edges.len()),
            retained_artifact_count: loop_state
                .latest_gatherer_trace
                .as_ref()
                .map(|planner| planner.retained_artifacts.len()),
        });

        if !used_workspace_resources && planner_stopped_without_resource_use(&loop_state) {
            return Ok(PlannerLoopOutcome {
                evidence: None,
                direct_answer: direct_answer.or_else(|| {
                    instruction_frame
                        .as_ref()
                        .filter(|frame| frame.has_pending_workspace_obligation())
                        .map(blocked_instruction_response)
                }),
                instruction_frame,
                grounding: context.grounding.clone(),
                continuation: None,
            });
        }

        Ok(PlannerLoopOutcome {
            evidence: Some(build_planner_evidence_bundle(
                &context.prepared,
                prompt,
                &loop_state,
                completed,
                &stop_reason,
            )),
            direct_answer: direct_answer.or_else(|| {
                instruction_frame
                    .as_ref()
                    .filter(|frame| frame.has_pending_workspace_obligation())
                    .map(blocked_instruction_response)
            }),
            instruction_frame,
            grounding: context.grounding.clone(),
            continuation: None,
        })
    }

    pub(super) fn expire_turn_control_requests(
        &self,
        trace: &Arc<StructuredTurnTrace>,
        detail: &str,
    ) {
        self.service.expire_turn_control_requests(trace, detail);
    }
}
