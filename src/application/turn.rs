//! Turn orchestration phase: top-level `process_prompt*` entry points that
//! drive interpretation, routing, agent loop execution, synthesis, and
//! transcript persistence for a single user turn. Free functions here
//! replace the prior `TurnOrchestrationChamber` wrapper struct.

use super::*;

pub(super) async fn process_prompt(service: &AgentRuntime, prompt: &str) -> Result<String> {
    let session = service.create_conversation_session();
    process_prompt_in_session_with_mode_request_and_sink(
        service,
        prompt,
        session,
        None,
        Arc::clone(&service.event_sink),
    )
    .await
}

pub(super) async fn process_prompt_with_sink(
    service: &AgentRuntime,
    prompt: &str,
    event_sink: Arc<dyn TurnEventSink>,
) -> Result<String> {
    let session = service.create_conversation_session();
    process_prompt_in_session_with_mode_request_and_sink(service, prompt, session, None, event_sink)
        .await
}

pub(super) async fn process_prompt_in_session_with_sink(
    service: &AgentRuntime,
    prompt: &str,
    session: ConversationSession,
    event_sink: Arc<dyn TurnEventSink>,
) -> Result<String> {
    process_prompt_in_session_with_mode_request_and_sink(service, prompt, session, None, event_sink)
        .await
}

pub(super) async fn process_prompt_in_session_with_mode_request_and_sink(
    service: &AgentRuntime,
    prompt: &str,
    session: ConversationSession,
    mode_request: Option<CollaborationModeRequest>,
    event_sink: Arc<dyn TurnEventSink>,
) -> Result<String> {
    let event_sink = service.wrap_sink_with_observers(event_sink);
    let mut current_prompt = prompt.to_string();
    let turn_contract = resolve_collaboration_mode_request(mode_request);

    loop {
        service.persist_prompt_history(&current_prompt);
        let runtime_guard = service.runtime.read().await;
        let runtime = runtime_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Turn runtime not initialized"))?;
        let prepared = runtime.prepared.clone();
        service.execution_hand_registry().set_governance_profile(
            prepared
                .harness_profile()
                .active_execution_governance()
                .clone(),
        );
        let action_selector = Arc::clone(&runtime.action_selector);
        let final_renderer = Arc::clone(&runtime.final_renderer);
        let retrieval_provider = runtime.retrieval_provider.clone();
        drop(runtime_guard);

        let interpretation = context_assembly::derive_interpretation_context(
            service,
            &current_prompt,
            action_selector.as_ref(),
            event_sink.clone(),
        )
        .await;
        let turn_id = session.allocate_turn_id();
        let active_thread = session.active_thread().thread_ref;
        let _active_turn = ActiveTurnGuard::new(session.clone(), turn_id.clone());
        let trace = Arc::new(StructuredTurnTrace::new(
            event_sink.clone(),
            Arc::clone(&service.trace_recorder),
            service.cloned_forensic_observers(),
            session.clone(),
            turn_id,
            active_thread.clone(),
        ));
        trace.record_turn_start(&current_prompt, &interpretation, &prepared);
        trace.emit(TurnEvent::CollaborationModeChanged {
            result: turn_contract.clone(),
        });
        let harness_profile = prepared.harness_profile();
        trace.emit(TurnEvent::ExecutionGovernanceProfileApplied {
            snapshot: crate::domain::model::ExecutionGovernanceSnapshot::new(
                harness_profile.requested.id(),
                harness_profile.active.id(),
                harness_profile.active_execution_governance().clone(),
            ),
        });
        conversation_read_model::emit_transcript_update(service, &session.task_id());
        trace.emit(TurnEvent::InterpretationContext {
            context: interpretation.clone(),
        });

        let action_selection_capability = action_selector.capability();
        trace.emit(TurnEvent::PlannerCapability {
            provider: prepared.planner.model_id.clone(),
            capability: format_action_selection_capability(&action_selection_capability),
        });

        let recent_turns =
            final_rendering::recent_turn_summaries(service, &session, final_renderer.as_ref())?;
        let specialist_runtime_notes = final_rendering::specialist_runtime_notes(
            service,
            &current_prompt,
            &session,
            &prepared,
        );
        let recent_thread_summary = session.recent_thread_summary(&active_thread);
        let controller_edit =
            controller_prompt_edit_instruction(&service.workspace_root, &current_prompt);
        let initial_edit =
            sanitize_initial_edit_instruction_for_turn_contract(&turn_contract, controller_edit);
        if initial_edit.known_edit && turn_contract.active.mutation_posture.allows_mutation() {
            let candidate_summary = if initial_edit.candidate_files.is_empty() {
                "no candidate files surfaced yet".to_string()
            } else {
                format!(
                    "candidate files: {}",
                    initial_edit.candidate_files.join(", ")
                )
            };
            trace.emit(TurnEvent::Fallback {
                stage: "action-bias".to_string(),
                reason: format!(
                    "controller inferred a concrete repository edit from the prompt and exposed workspace editor pressure to the agent loop; {candidate_summary}"
                ),
            });
        }
        let mut instruction_frame = instruction_frame_from_initial_edit(&initial_edit);
        if turn_contract.active.mutation_posture.allows_mutation() {
            instruction_frame = merge_instruction_frames(
                instruction_frame,
                controller_prompt_commit_instruction(&current_prompt),
            );
        }
        let grounding = initial_loop_grounding_requirement(&turn_contract, &current_prompt);

        let (agent_loop_outcome, route_summary) = match action_selection_capability {
            PlannerCapability::Available => {
                let resolver: Arc<dyn ContextResolver> = Arc::new(TransitContextResolver::new(
                    Arc::clone(&service.trace_recorder),
                ));

                let outcome = agent_loop::execute_agent_loop(
                    service,
                    &current_prompt,
                    AgentLoopContext {
                        prepared: prepared.clone(),
                        action_selector,
                        retrieval_provider,
                        resolver,
                        entity_resolver: Arc::clone(&service.entity_resolver),
                        workspace_capability_surface: service
                            .workspace_action_executor()
                            .capability_surface(),
                        execution_hands: service.execution_hand_diagnostics(),
                        governance_profile: service.execution_hand_registry().governance_profile(),
                        external_capabilities: service.external_capability_descriptors(),
                        interpretation: interpretation.clone(),
                        operator_memory: service
                            .operator_memory
                            .operator_memory_documents(&service.workspace_root),
                        recent_turns: recent_turns.clone(),
                        recent_thread_summary: recent_thread_summary.clone(),
                        turn_contract: turn_contract.clone(),
                        specialist_runtime_notes,
                        instruction_frame,
                        initial_edit,
                        grounding,
                    },
                    None,
                    Arc::clone(&trace),
                )
                .await?;
                let route_summary = if outcome.direct_answer.is_some() {
                    "agent loop selected a terminal direct response".to_string()
                } else if outcome.evidence.is_some() {
                    format!(
                        "agent loop gathered evidence with action-selection client '{}' before final-rendering client '{}'",
                        prepared.planner.model_id, prepared.synthesizer.model_id
                    )
                } else {
                    format!(
                        "agent loop completed without gathered evidence before final-rendering client '{}'",
                        prepared.synthesizer.model_id
                    )
                };
                (outcome, route_summary)
            }
            PlannerCapability::Unsupported { reason } => {
                let route_summary = format!(
                    "action-selection client '{}' is unavailable, so the turn will fall back to final-rendering client '{}' for a direct response",
                    prepared.planner.model_id, prepared.synthesizer.model_id
                );
                trace.emit(TurnEvent::Fallback {
                    stage: "action-selection".to_string(),
                    reason: format!("action-selection client unavailable before agent loop action selection: {reason}"),
                });
                (
                    AgentLoopOutcome {
                        evidence: None,
                        direct_answer: None,
                        instruction_frame: None,
                        grounding: None,
                        continuation: None,
                    },
                    route_summary,
                )
            }
        };

        agent_loop::expire_turn_control_requests(
            service,
            &trace,
            "The turn closed before the requested control could reach another safe checkpoint.",
        );

        let intent = if agent_loop_outcome.evidence.is_some() {
            TurnIntent::Planned
        } else {
            TurnIntent::DirectResponse
        };
        trace.emit(TurnEvent::IntentClassified {
            intent: intent.clone(),
        });
        trace.emit(TurnEvent::RouteSelected {
            summary: route_summary,
        });

        if let Some(continuation) = agent_loop_outcome.continuation {
            trace.record_checkpoint_without_response(
                TraceCheckpointKind::TurnCompleted,
                continuation.summary,
            );
            current_prompt = continuation.prompt;
            continue;
        }

        if let Some(reply) = agent_loop_outcome.direct_answer {
            let response = if let Some(frame) = agent_loop_outcome
                .instruction_frame
                .as_ref()
                .filter(|frame| frame.has_pending_workspace_obligation())
            {
                blocked_instruction_response(frame)
            } else {
                reply
            };
            trace.emit(TurnEvent::SynthesisReady {
                grounded: false,
                citations: Vec::new(),
                insufficient_evidence: false,
            });
            let reply = final_rendering::finalize_turn_response(
                service,
                &trace,
                &session,
                &active_thread,
                &current_prompt,
                &response,
            );
            return Ok(reply);
        }

        let engine = final_renderer;
        let trace_for_reply = Arc::clone(&trace);
        let event_sink = trace.as_event_sink();
        let session_for_reply = session.clone();
        let thread_for_reply = active_thread;
        let prompt_for_model = current_prompt.clone();
        let handoff = FinalRenderingHandoff {
            recent_turns,
            recent_thread_summary,
            turn_contract: turn_contract.clone(),
            instruction_frame: agent_loop_outcome.instruction_frame.clone(),
            grounding: agent_loop_outcome.grounding.clone(),
        };
        if let Some(frame) = agent_loop_outcome
            .instruction_frame
            .as_ref()
            .filter(|frame| frame.has_pending_workspace_obligation())
        {
            let response = blocked_instruction_response(frame);
            trace.emit(TurnEvent::SynthesisReady {
                grounded: false,
                citations: Vec::new(),
                insufficient_evidence: false,
            });
            let reply = final_rendering::finalize_turn_response(
                service,
                &trace_for_reply,
                &session_for_reply,
                &thread_for_reply,
                &current_prompt,
                &response,
            );
            return Ok(reply);
        }
        let reply = tokio::task::spawn_blocking(move || {
            engine.respond_for_turn(
                &prompt_for_model,
                intent,
                agent_loop_outcome.evidence.as_ref(),
                &handoff,
                event_sink,
            )
        })
        .await
        .map_err(|err| anyhow::anyhow!("Sift session task failed: {err}"))??;
        let response = AuthoredResponse::from_plain_text(
            trace_for_reply.completion_response_mode_for_synthesis(
                agent_loop_outcome.instruction_frame.as_ref(),
            ),
            &reply,
        );
        let reply = final_rendering::finalize_turn_response(
            service,
            &trace_for_reply,
            &session_for_reply,
            &thread_for_reply,
            &current_prompt,
            &response,
        );
        return Ok(reply);
    }
}

pub(super) async fn process_thread_candidate_in_session_with_sink(
    service: &AgentRuntime,
    candidate: ThreadCandidate,
    session: ConversationSession,
    event_sink: Arc<dyn TurnEventSink>,
) -> Result<String> {
    let event_sink = service.wrap_sink_with_observers(event_sink);
    let runtime_guard = service.runtime.read().await;
    let runtime = runtime_guard
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Turn runtime not initialized"))?;
    let action_selector = Arc::clone(&runtime.action_selector);
    let final_renderer = Arc::clone(&runtime.final_renderer);
    drop(runtime_guard);

    let interpretation = context_assembly::derive_interpretation_context(
        service,
        &candidate.prompt,
        action_selector.as_ref(),
        event_sink.clone(),
    )
    .await;
    let source_thread = candidate.active_thread.clone();
    let turn_id = session.allocate_turn_id();
    let trace = Arc::new(StructuredTurnTrace::new(
        event_sink,
        Arc::clone(&service.trace_recorder),
        service.cloned_forensic_observers(),
        session.clone(),
        turn_id,
        source_thread.clone(),
    ));
    trace.emit(TurnEvent::ThreadCandidateCaptured {
        candidate_id: candidate.candidate_id.as_str().to_string(),
        active_thread: candidate.active_thread.stable_id(),
        prompt: candidate.prompt.clone(),
    });
    trace.record_thread_candidate(&candidate);

    let recent_turns =
        final_rendering::recent_turn_summaries(service, &session, final_renderer.as_ref())?;
    let active_thread = session.active_thread();
    let thread_request = ThreadDecisionRequest::new(
        service.workspace_root.clone(),
        interpretation,
        active_thread.clone(),
        candidate.clone(),
    )
    .with_recent_turns(recent_turns)
    .with_known_threads(session.known_threads())
    .with_recent_thread_summary(session.recent_thread_summary(&active_thread.thread_ref));

    let decision = action_selector
        .select_thread_decision(&thread_request, trace.clone() as Arc<dyn TurnEventSink>)
        .await?;
    trace.emit(TurnEvent::ThreadDecisionApplied {
        candidate_id: candidate.candidate_id.as_str().to_string(),
        decision: decision.kind.label().to_string(),
        target_thread: decision.target_thread.stable_id(),
        rationale: decision.rationale.clone(),
    });
    trace.record_thread_decision(&decision, &source_thread);

    let branch_id = if matches!(decision.kind, ThreadDecisionKind::OpenChildThread) {
        let branch_id = session.next_branch_id();
        trace.declare_branch(
            branch_id.clone(),
            decision
                .new_thread_label
                .as_deref()
                .unwrap_or(candidate.prompt.as_str()),
            Some(decision.rationale.as_str()),
            source_thread.branch_id(),
        );
        Some(branch_id)
    } else {
        None
    };

    if matches!(decision.kind, ThreadDecisionKind::MergeIntoTarget) {
        trace.emit(TurnEvent::ThreadMerged {
            source_thread: source_thread.stable_id(),
            target_thread: decision.target_thread.stable_id(),
            mode: decision
                .merge_mode
                .unwrap_or(ThreadMergeMode::Summary)
                .label()
                .to_string(),
            summary: decision.merge_summary.clone(),
        });
        trace.record_thread_merge(&decision, &source_thread, &decision.target_thread);
    }

    session.apply_thread_decision(&decision, branch_id, &candidate.prompt);
    process_prompt_in_session_with_sink(
        service,
        &candidate.prompt,
        session,
        trace.downstream.clone(),
    )
    .await
}
