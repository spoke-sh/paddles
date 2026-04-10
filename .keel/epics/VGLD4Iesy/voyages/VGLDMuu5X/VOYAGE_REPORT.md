# VOYAGE REPORT: Decouple Brain From Hands In The Local Runtime

## Voyage Metadata
- **ID:** VGLDMuu5X
- **Epic:** VGLD4Iesy
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Define Shared Hand Lifecycle And Diagnostics Surface
- **ID:** VGLDQ9Zoe
- **Status:** done

#### Summary
Define the common execution-hand contract that local action surfaces should share. This story should name the lifecycle, provisioning, execution, recovery, and diagnostics vocabulary before adapters are migrated onto it.

#### Acceptance Criteria
- [x] The runtime defines a shared hand lifecycle and diagnostics surface that covers local execution boundaries consistently [SRS-01/AC-01] <!-- verify: cargo test service_new_exposes_default_execution_hand_diagnostics_surface -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] The hand vocabulary is explicit enough that later workspace, terminal, and transport stories can adopt it without inventing new state names [SRS-01/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "Hands stay explicit|ExecutionHandBoundary|Execution Hand Contract|Execution Hand Registry|workspace_editor|terminal_runner|transport_mediator|described|provisioning|ready|executing|recovering|degraded|failed" README.md ARCHITECTURE.md CONFIGURATION.md src/domain/model/execution_hand.rs src/infrastructure/execution_hand.rs', SRS-01:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGLDQ9Zoe/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGLDQ9Zoe/EVIDENCE/ac-2.log)

### Adapt Workspace And Terminal Execution To Hand Contracts
- **ID:** VGLDQAMpx
- **Status:** done

#### Summary
Adapt the local workspace editor and terminal runner to the shared hand contract. This story should preserve current authored-workspace and shell semantics while shifting them onto the common execution interface.

#### Acceptance Criteria
- [x] Workspace editing and terminal execution run through the shared hand contract without breaking current local-first operator behavior [SRS-02/AC-01] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test workspace_editor_reports_hand_execution_diagnostics_after_successful_write -- --nocapture && cargo test terminal_runner_reports_hand_execution_diagnostics_after_command_completion -- --nocapture', SRS-02:start:end, proof: ac-1.log-->
- [x] Hand execution remains observable through the existing runtime trace and diagnostics surfaces after the adapter migration [SRS-02/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test shared_bootstrap_route_returns_shared_session_projection_and_execution_hand_diagnostics -- --nocapture && cargo test health_route_reports_execution_hand_and_native_transport_diagnostics -- --nocapture', SRS-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGLDQAMpx/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGLDQAMpx/EVIDENCE/ac-2.log)

### Isolate Credentials Behind Transport And Tool Mediators
- **ID:** VGLDQApqs
- **Status:** done

#### Summary
Introduce mediated credential boundaries for local hands that interact with privileged transport or tool state. This story should push secrets farther away from generated code and shell execution.

#### Acceptance Criteria
- [x] Privileged transport and tool credentials are mediated so local execution hands do not receive more authority than required [SRS-03/AC-01] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test terminal_runner_does_not_forward_provider_or_transport_credentials_into_shells -- --nocapture && cargo test mediator_collects_provider_and_native_transport_secret_env_vars -- --nocapture', SRS-03:start:end, proof: ac-1.log -->
- [x] Failure and degradation paths for mediated credential access remain explicit in runtime diagnostics and traces [SRS-03/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test missing_bearer_token_env_marks_transport_mediator_failed_in_runtime_diagnostics -- --nocapture && cargo test mediator_reports_failed_transport_diagnostics_when_bearer_token_is_missing -- --nocapture', SRS-03:start:end, proof: ac-2.log -->


