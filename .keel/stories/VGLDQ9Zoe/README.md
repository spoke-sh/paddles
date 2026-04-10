---
# system-managed
id: VGLDQ9Zoe
status: done
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T17:34:09
# authored
title: Define Shared Hand Lifecycle And Diagnostics Surface
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMuu5X
index: 1
started_at: 2026-04-09T17:25:49
completed_at: 2026-04-09T17:34:09
---

# Define Shared Hand Lifecycle And Diagnostics Surface

## Summary

Define the common execution-hand contract that local action surfaces should share. This story should name the lifecycle, provisioning, execution, recovery, and diagnostics vocabulary before adapters are migrated onto it.

## Acceptance Criteria

- [x] The runtime defines a shared hand lifecycle and diagnostics surface that covers local execution boundaries consistently [SRS-01/AC-01] <!-- verify: cargo test service_new_exposes_default_execution_hand_diagnostics_surface -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] The hand vocabulary is explicit enough that later workspace, terminal, and transport stories can adopt it without inventing new state names [SRS-01/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "Hands stay explicit|ExecutionHandBoundary|Execution Hand Contract|Execution Hand Registry|workspace_editor|terminal_runner|transport_mediator|described|provisioning|ready|executing|recovering|degraded|failed" README.md ARCHITECTURE.md CONFIGURATION.md src/domain/model/execution_hand.rs src/infrastructure/execution_hand.rs', SRS-01:start:end, proof: ac-2.log-->
