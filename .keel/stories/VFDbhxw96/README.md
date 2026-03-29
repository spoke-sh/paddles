---
# system-managed
id: VFDbhxw96
status: in-progress
created_at: 2026-03-28T19:12:55
updated_at: 2026-03-28T19:15:10
# authored
title: Add Interactive Terminal Runtime And Transcript State
type: feat
operator-signal:
scope: VFDbdzqtU/VFDbfLe0E
index: 1
started_at: 2026-03-28T19:15:10
---

# Add Interactive Terminal Runtime And Transcript State

## Summary

Add the terminal runtime and transcript state needed to replace the legacy
interactive stdin loop with a dedicated TUI while keeping one-shot mode plain.

## Acceptance Criteria

- [ ] Interactive mode enters a dedicated TUI runtime with clean terminal setup/teardown, while `--prompt` continues to use the plain stdout path. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] The TUI owns transcript/app state for at least user rows, assistant rows, action/event rows, and composer input. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] Tests or transcript proofs cover terminal runtime behavior and one-shot-path preservation. [SRS-01/AC-03] <!-- verify: manual, SRS-01:start:end -->
