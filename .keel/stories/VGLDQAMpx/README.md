---
# system-managed
id: VGLDQAMpx
status: done
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T17:45:15
# authored
title: Adapt Workspace And Terminal Execution To Hand Contracts
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMuu5X
index: 2
started_at: 2026-04-09T17:35:04
completed_at: 2026-04-09T17:45:15
---

# Adapt Workspace And Terminal Execution To Hand Contracts

## Summary

Adapt the local workspace editor and terminal runner to the shared hand contract. This story should preserve current authored-workspace and shell semantics while shifting them onto the common execution interface.

## Acceptance Criteria

- [x] Workspace editing and terminal execution run through the shared hand contract without breaking current local-first operator behavior [SRS-02/AC-01] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test workspace_editor_reports_hand_execution_diagnostics_after_successful_write -- --nocapture && cargo test terminal_runner_reports_hand_execution_diagnostics_after_command_completion -- --nocapture', SRS-02:start:end, proof: ac-1.log-->
- [x] Hand execution remains observable through the existing runtime trace and diagnostics surfaces after the adapter migration [SRS-02/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test shared_bootstrap_route_returns_shared_session_projection_and_execution_hand_diagnostics -- --nocapture && cargo test health_route_reports_execution_hand_and_native_transport_diagnostics -- --nocapture', SRS-02:start:end, proof: ac-2.log-->
