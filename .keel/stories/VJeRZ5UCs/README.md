---
# system-managed
id: VJeRZ5UCs
status: backlog
created_at: 2026-05-14T19:17:30
updated_at: 2026-05-14T19:19:31
# authored
title: Remove Pre-Loop Bootstrap Overrides
type: refactor
operator-signal:
scope: VJeQx1O20/VJeRAR1IS
index: 1
---

# Remove Pre-Loop Bootstrap Overrides

## Summary

Remove the controller bootstrap overrides that currently replace the model's first action before the agent loop runs, converting any still-needed pressure into loop-visible signals.

## Acceptance Criteria

- [ ] Commit, known-edit, repository-grounding, and review bootstrap helpers no longer force initial actions before the loop. [SRS-01/AC-01] <!-- verify: rg -n "bootstrap_.*initial_action|known-edit-bootstrap|commit-bootstrap|grounding-bootstrap|review-bootstrap" src/application, SRS-01:start:end -->
- [ ] Remaining edit/commit/grounding/review pressure is visible in loop request state or execution contract lines. [SRS-01/AC-02] <!-- verify: cargo test bootstrap_pressure_is_loop_visible_context -- --nocapture, SRS-01:start:end -->
- [ ] Removing bootstraps does not weaken mutation or commit completion enforcement. [SRS-NFR-01/AC-03] <!-- verify: cargo test edit_commit_boundaries_survive_bootstrap_removal -- --nocapture, SRS-NFR-01:start:end -->
