# Transcript-Driven Interactive Terminal - SRS

## Summary

Epic: VFDbdzqtU
Goal: Replace the plain interactive loop with a Codex-style TUI that renders styled user, assistant, and action transcript cells while preserving paddles routing and one-shot CLI behavior.

## Scope

### In Scope

- [SCOPE-01] Add terminal lifecycle and transcript state for interactive mode.
- [SCOPE-02] Render styled transcript rows for user, assistant, and action/event output.
- [SCOPE-03] Bridge live paddles turn events and progressive assistant rendering into the TUI.
- [SCOPE-04] Preserve plain one-shot CLI behavior and document/prove the interactive UX.

### Out of Scope

- [SCOPE-05] Reworking paddles model routing or gatherer logic beyond what is needed to surface existing events in the UI.
- [SCOPE-06] Importing Codex’s full multi-pane app, voice input, or remote-only UI features.
- [SCOPE-07] Adding real token streaming from model runtimes that do not currently expose it.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Interactive mode must enter a dedicated terminal UI runtime with transcript state, while `--prompt` keeps the non-TUI output path. | SCOPE-01, SCOPE-04 | FR-01 | manual |
| SRS-02 | The transcript model must represent at least user turns, assistant turns, and action/event rows so the UI can render each with distinct semantics. | SCOPE-01, SCOPE-02 | FR-02 | manual |
| SRS-03 | The TUI renderer must visually distinguish user, assistant, and action/event rows with stable styles that stay readable against the terminal background. | SCOPE-02 | FR-02 | manual |
| SRS-04 | Live controller turn events must flow into the interactive transcript as execution progresses instead of appearing only as raw stdout logs. | SCOPE-03 | FR-03 | manual |
| SRS-05 | Final assistant answers must appear progressively in the transcript and preserve the final grounded/cited content returned by paddles. | SCOPE-03 | FR-04 | manual |
| SRS-06 | Foundational docs and proof artifacts must describe the new interactive UX, the one-shot/plain distinction, and the expected transcript shape. | SCOPE-04 | FR-06 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The interactive TUI must add no mandatory network dependency and must remain local-first. | SCOPE-01, SCOPE-04 | NFR-01 | manual |
| SRS-NFR-02 | Terminal raw-mode and screen state must restore cleanly on exit and on common error paths. | SCOPE-01 | NFR-02 | manual |
| SRS-NFR-03 | The TUI architecture must stay small and paddles-owned, using terminal libraries rather than importing a large external app shell wholesale. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
