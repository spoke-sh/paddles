---
# system-managed
id: VGLDQApqs
status: done
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T18:13:32
# authored
title: Isolate Credentials Behind Transport And Tool Mediators
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMuu5X
index: 3
started_at: 2026-04-09T18:01:30
completed_at: 2026-04-09T18:13:32
---

# Isolate Credentials Behind Transport And Tool Mediators

## Summary

Introduce mediated credential boundaries for local hands that interact with privileged transport or tool state. This story should push secrets farther away from generated code and shell execution.

## Acceptance Criteria

- [x] Privileged transport and tool credentials are mediated so local execution hands do not receive more authority than required [SRS-03/AC-01] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test terminal_runner_does_not_forward_provider_or_transport_credentials_into_shells -- --nocapture && cargo test mediator_collects_provider_and_native_transport_secret_env_vars -- --nocapture', SRS-03:start:end, proof: ac-1.log -->
- [x] Failure and degradation paths for mediated credential access remain explicit in runtime diagnostics and traces [SRS-03/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test missing_bearer_token_env_marks_transport_mediator_failed_in_runtime_diagnostics -- --nocapture && cargo test mediator_reports_failed_transport_diagnostics_when_bearer_token_is_missing -- --nocapture', SRS-03:start:end, proof: ac-2.log -->
