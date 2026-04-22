use super::*;

pub(super) struct TurnOrchestrationChamber<'a> {
    service: &'a MechSuitService,
}

impl<'a> TurnOrchestrationChamber<'a> {
    pub(super) const fn new(service: &'a MechSuitService) -> Self {
        Self { service }
    }

    pub(super) async fn process_prompt(&self, prompt: &str) -> Result<String> {
        let session = self.service.create_conversation_session();
        self.process_prompt_in_session_with_mode_request_and_sink(
            prompt,
            session,
            None,
            Arc::clone(&self.service.event_sink),
        )
        .await
    }

    pub(super) async fn process_prompt_with_sink(
        &self,
        prompt: &str,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        let session = self.service.create_conversation_session();
        self.process_prompt_in_session_with_mode_request_and_sink(prompt, session, None, event_sink)
            .await
    }

    pub(super) async fn process_prompt_in_session_with_sink(
        &self,
        prompt: &str,
        session: ConversationSession,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        self.process_prompt_in_session_with_mode_request_and_sink(prompt, session, None, event_sink)
            .await
    }

    pub(super) async fn process_prompt_in_session_with_mode_request_and_sink(
        &self,
        prompt: &str,
        session: ConversationSession,
        mode_request: Option<CollaborationModeRequest>,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        let event_sink = self.service.wrap_sink_with_observers(event_sink);
        let mut current_prompt = prompt.to_string();
        let collaboration = resolve_collaboration_mode_request(mode_request);

        loop {
            self.service.persist_prompt_history(&current_prompt);
            let runtime_guard = self.service.runtime.read().await;
            let runtime = runtime_guard
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Runtime lanes not initialized"))?;
            let prepared = runtime.prepared.clone();
            self.service
                .execution_hand_registry()
                .set_governance_profile(
                    prepared
                        .harness_profile()
                        .active_execution_governance()
                        .clone(),
                );
            let planner_engine = Arc::clone(&runtime.planner_engine);
            let synthesizer_engine = Arc::clone(&runtime.synthesizer_engine);
            let gatherer = runtime.gatherer.clone();
            drop(runtime_guard);

            let interpretation = self
                .service
                .interpretation_chamber()
                .derive_interpretation_context(
                    &current_prompt,
                    planner_engine.as_ref(),
                    event_sink.clone(),
                )
                .await;
            let turn_id = session.allocate_turn_id();
            let active_thread = session.active_thread().thread_ref;
            let _active_turn = ActiveTurnGuard::new(session.clone(), turn_id.clone());
            let trace = Arc::new(StructuredTurnTrace::new(
                event_sink.clone(),
                Arc::clone(&self.service.trace_recorder),
                self.service.cloned_forensic_observers(),
                session.clone(),
                turn_id,
                active_thread.clone(),
            ));
            trace.record_turn_start(&current_prompt, &interpretation, &prepared);
            trace.emit(TurnEvent::CollaborationModeChanged {
                result: collaboration.clone(),
            });
            let harness_profile = prepared.harness_profile();
            trace.emit(TurnEvent::ExecutionGovernanceProfileApplied {
                snapshot: crate::domain::model::ExecutionGovernanceSnapshot::new(
                    harness_profile.requested.id(),
                    harness_profile.active.id(),
                    harness_profile.active_execution_governance().clone(),
                ),
            });
            self.service
                .conversation_read_model()
                .emit_transcript_update(&session.task_id());
            trace.emit(TurnEvent::InterpretationContext {
                context: interpretation.clone(),
            });

            let planner_capability = planner_engine.capability();
            trace.emit(TurnEvent::PlannerCapability {
                provider: prepared.planner.model_id.clone(),
                capability: format_planner_capability(&planner_capability),
            });

            let recent_turns = self
                .service
                .synthesis_chamber()
                .recent_turn_summaries(&session, synthesizer_engine.as_ref())?;
            let specialist_runtime_notes = self
                .service
                .synthesis_chamber()
                .specialist_runtime_notes(&current_prompt, &session, &prepared);
            let recent_thread_summary = session.recent_thread_summary(&active_thread);
            let request = PlannerRequest::new(
                &current_prompt,
                self.service.workspace_root.clone(),
                interpretation.clone(),
                PlannerBudget::default(),
            )
            .with_collaboration(collaboration.clone())
            .with_recent_turns(recent_turns.clone())
            .with_recent_thread_summary(recent_thread_summary.clone())
            .with_runtime_notes(planner_runtime_notes(
                gatherer.as_ref(),
                &specialist_runtime_notes,
                &collaboration,
            ))
            .with_entity_resolver(Arc::clone(&self.service.entity_resolver));

            let execution_plan = match planner_capability {
                PlannerCapability::Available => {
                    let mut decision = planner_engine
                        .select_initial_action(&request, trace.clone() as Arc<dyn TurnEventSink>)
                        .await?;
                    let controller_edit = controller_prompt_edit_instruction(
                        &self.service.workspace_root,
                        &current_prompt,
                    );
                    let provider_edit_missing =
                        !decision.edit.known_edit && controller_edit.known_edit;
                    decision.edit =
                        merge_initial_edit_instruction(&decision.edit, &controller_edit);
                    if provider_edit_missing
                        && collaboration.active.mutation_posture.allows_mutation()
                    {
                        let candidate_summary = if decision.edit.candidate_files.is_empty() {
                            "no candidate files surfaced yet".to_string()
                        } else {
                            format!(
                                "candidate files: {}",
                                decision.edit.candidate_files.join(", ")
                            )
                        };
                        trace.emit(TurnEvent::Fallback {
                            stage: "action-bias".to_string(),
                            reason: format!(
                                "controller inferred a concrete repository edit from the prompt and activated workspace editor pressure; {candidate_summary}"
                            ),
                        });
                    }
                    let controller_commit_instruction =
                        controller_prompt_commit_instruction(&current_prompt);
                    if collaboration.active.mutation_posture.allows_mutation()
                        && let Some(bootstrapped) =
                            bootstrap_git_commit_initial_action(&current_prompt, &decision)
                    {
                        trace.emit(TurnEvent::Fallback {
                            stage: "commit-bootstrap".to_string(),
                            reason: format!(
                                "commit-oriented turn bypassed initial `{}` and forced `{}` to inspect workspace status before committing",
                                decision.action.summary(),
                                bootstrapped.action.summary()
                            ),
                        });
                        decision = bootstrapped;
                    }
                    if let Some(bootstrapped) = self
                        .service
                        .interpretation_chamber()
                        .bootstrap_known_edit_initial_action(
                            &current_prompt,
                            &interpretation,
                            &recent_turns,
                            gatherer.as_ref(),
                            &decision,
                            trace.as_ref(),
                        )
                        .await?
                    {
                        let candidate_summary = if bootstrapped.edit.candidate_files.is_empty() {
                            "no viable candidates discovered".to_string()
                        } else {
                            format!(
                                "candidate files: {}",
                                bootstrapped.edit.candidate_files.join(", ")
                            )
                        };
                        trace.emit(TurnEvent::Fallback {
                            stage: "known-edit-bootstrap".to_string(),
                            reason: format!(
                                "known edit turn bypassed initial `{}` and forced `{}`; {}",
                                decision.action.summary(),
                                bootstrapped.action.summary(),
                                candidate_summary
                            ),
                        });
                        decision = bootstrapped;
                    }
                    if let Some(bootstrapped) =
                        bootstrap_repository_grounding_initial_action(&current_prompt, &decision)
                    {
                        trace.emit(TurnEvent::Fallback {
                            stage: "grounding-bootstrap".to_string(),
                            reason: format!(
                                "repo-scoped conversational turn bypassed initial `{}` and forced `{}` to ground the reply locally",
                                decision.action.summary(),
                                bootstrapped.action.summary()
                            ),
                        });
                        decision = bootstrapped;
                    }
                    if collaboration.active.mode == CollaborationMode::Review
                        && let Some(bootstrapped) = bootstrap_review_initial_action(&decision)
                    {
                        trace.emit(TurnEvent::Fallback {
                            stage: "review-bootstrap".to_string(),
                            reason: format!(
                                "review mode bypassed initial `{}` and forced `{}` to inspect local changes before synthesis",
                                decision.action.summary(),
                                bootstrapped.action.summary()
                            ),
                        });
                        decision = bootstrapped;
                    }
                    decision = sanitize_initial_action_decision_for_collaboration(
                        &collaboration,
                        decision,
                    );
                    trace.emit(TurnEvent::PlannerActionSelected {
                        sequence: 1,
                        action: decision.action.summary(),
                        rationale: decision.rationale.clone(),
                    });
                    trace.record_planner_action(
                        &decision.action.summary(),
                        &decision.rationale,
                        None,
                    );
                    let mut execution_plan =
                        execution_plan_from_initial_action(&prepared, decision);
                    if collaboration.active.mutation_posture.allows_mutation() {
                        execution_plan.instruction_frame = merge_instruction_frames(
                            execution_plan.instruction_frame.clone(),
                            controller_commit_instruction,
                        );
                    }
                    execution_plan
                }
                PlannerCapability::Unsupported { reason } => {
                    trace.emit(TurnEvent::Fallback {
                        stage: "planner".to_string(),
                        reason: format!(
                            "planner unavailable before first action selection: {reason}"
                        ),
                    });
                    fallback_execution_plan(&prepared)
                }
            };

            let mut execution_checklist =
                build_execution_checklist(&current_prompt, &recent_turns, &execution_plan);

            trace.emit(TurnEvent::IntentClassified {
                intent: execution_plan.intent.clone(),
            });
            trace.emit(TurnEvent::RouteSelected {
                summary: execution_plan.route_summary.clone(),
            });
            if let Some(checklist) = execution_checklist.as_mut() {
                checklist.emit(trace.as_ref());
            }

            let planner_outcome = match execution_plan.path {
                PromptExecutionPath::PlannerThenSynthesize => {
                    let recent_turns = self
                        .service
                        .synthesis_chamber()
                        .recent_turn_summaries(&session, synthesizer_engine.as_ref())?;

                    let resolver: Arc<dyn ContextResolver> = if let Some(transit) = self
                        .service
                        .trace_recorder
                        .as_any()
                        .downcast_ref::<TransitTraceRecorder>()
                    {
                        Arc::new(TransitContextResolver::new(Arc::new(transit.clone())))
                    } else {
                        Arc::new(NoopContextResolver)
                    };

                    self.service
                        .recursive_control()
                        .execute_recursive_planner_loop(
                            &current_prompt,
                            PlannerLoopContext {
                                prepared: prepared.clone(),
                                planner_engine,
                                gatherer,
                                resolver,
                                entity_resolver: Arc::clone(&self.service.entity_resolver),
                                interpretation: interpretation.clone(),
                                recent_turns,
                                recent_thread_summary: recent_thread_summary.clone(),
                                collaboration: collaboration.clone(),
                                specialist_runtime_notes,
                                instruction_frame: execution_plan.instruction_frame.clone(),
                                initial_edit: execution_plan.initial_edit.clone(),
                                grounding: execution_plan.grounding.clone(),
                            },
                            execution_plan.initial_planner_decision.clone(),
                            execution_checklist,
                            Arc::clone(&trace),
                        )
                        .await?
                }
                PromptExecutionPath::SynthesizerOnly => PlannerLoopOutcome {
                    evidence: None,
                    direct_answer: execution_plan.direct_answer.clone(),
                    instruction_frame: execution_plan.instruction_frame.clone(),
                    grounding: execution_plan.grounding.clone(),
                    continuation: None,
                },
            };

            self.service.recursive_control().expire_turn_control_requests(
                &trace,
                "The turn closed before the requested control could reach another safe checkpoint.",
            );

            if let Some(continuation) = planner_outcome.continuation {
                trace.record_checkpoint_without_response(
                    TraceCheckpointKind::TurnCompleted,
                    continuation.summary,
                );
                current_prompt = continuation.prompt;
                continue;
            }

            if let Some(reply) = planner_outcome.direct_answer {
                let response = if let Some(frame) = planner_outcome
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
                let reply = self.service.synthesis_chamber().finalize_turn_response(
                    &trace,
                    &session,
                    &active_thread,
                    &current_prompt,
                    &response,
                );
                return Ok(reply);
            }

            let intent = execution_plan.intent;
            let engine = synthesizer_engine;
            let trace_for_reply = Arc::clone(&trace);
            let event_sink = trace.as_event_sink();
            let session_for_reply = session.clone();
            let thread_for_reply = active_thread;
            let prompt_for_model = current_prompt.clone();
            let handoff = SynthesisHandoff {
                recent_turns,
                recent_thread_summary,
                collaboration: collaboration.clone(),
                instruction_frame: planner_outcome.instruction_frame.clone(),
                grounding: planner_outcome.grounding.clone(),
            };
            if let Some(frame) = planner_outcome
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
                let reply = self.service.synthesis_chamber().finalize_turn_response(
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
                    planner_outcome.evidence.as_ref(),
                    &handoff,
                    event_sink,
                )
            })
            .await
            .map_err(|err| anyhow::anyhow!("Sift session task failed: {err}"))??;
            let response = AuthoredResponse::from_plain_text(
                trace_for_reply.completion_response_mode_for_synthesis(
                    planner_outcome.instruction_frame.as_ref(),
                ),
                &reply,
            );
            let reply = self.service.synthesis_chamber().finalize_turn_response(
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
        &self,
        candidate: ThreadCandidate,
        session: ConversationSession,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        let event_sink = self.service.wrap_sink_with_observers(event_sink);
        let runtime_guard = self.service.runtime.read().await;
        let runtime = runtime_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime lanes not initialized"))?;
        let planner_engine = Arc::clone(&runtime.planner_engine);
        let synthesizer_engine = Arc::clone(&runtime.synthesizer_engine);
        drop(runtime_guard);

        let interpretation = self
            .service
            .interpretation_chamber()
            .derive_interpretation_context(
                &candidate.prompt,
                planner_engine.as_ref(),
                event_sink.clone(),
            )
            .await;
        let source_thread = candidate.active_thread.clone();
        let turn_id = session.allocate_turn_id();
        let trace = Arc::new(StructuredTurnTrace::new(
            event_sink,
            Arc::clone(&self.service.trace_recorder),
            self.service.cloned_forensic_observers(),
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

        let recent_turns = self
            .service
            .synthesis_chamber()
            .recent_turn_summaries(&session, synthesizer_engine.as_ref())?;
        let active_thread = session.active_thread();
        let thread_request = ThreadDecisionRequest::new(
            self.service.workspace_root.clone(),
            interpretation,
            active_thread.clone(),
            candidate.clone(),
        )
        .with_recent_turns(recent_turns)
        .with_known_threads(session.known_threads())
        .with_recent_thread_summary(session.recent_thread_summary(&active_thread.thread_ref));

        let decision = planner_engine
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
        self.process_prompt_in_session_with_sink(
            &candidate.prompt,
            session,
            trace.downstream.clone(),
        )
        .await
    }
}
