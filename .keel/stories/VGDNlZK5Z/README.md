---
# system-managed
id: VGDNlZK5Z
status: backlog
created_at: 2026-04-08T08:46:25
updated_at: 2026-04-08T08:50:22
# authored
title: Use Deterministic Resolution Before Edit State Actions
type: feat
operator-signal:
scope: VGDNcabks/VGDNh30T9
index: 1
---

# Use Deterministic Resolution Before Edit State Actions

## Summary

Use deterministic resolution in known-edit bootstrap and execution-pressure gates so edit-oriented turns validate likely targets before they read the wrong file or jump into placeholder patch mode.

## Acceptance Criteria

- [ ] Known-edit bootstrap consults deterministic resolution before broad search once a likely target family exists. [SRS-01/AC-01] <!-- verify: cargo nextest run known_edit_bootstrap_uses_deterministic_resolution --no-tests pass, SRS-01:start:end -->
- [ ] Execution-pressure reviews promote resolved targets into read/diff/edit actions instead of repeating broad search or inspect loops. [SRS-02/AC-02] <!-- verify: cargo nextest run execution_pressure_prefers_resolved_targets_over_repeated_search --no-tests pass, SRS-02:start:end -->
