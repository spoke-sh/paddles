---
# system-managed
id: VFfvAzX7H
status: in-progress
created_at: 2026-04-02T15:25:52
updated_at: 2026-04-02T15:28:04
# authored
title: Create Turborepo Workspace And Shared Frontend Scripts
type: feat
operator-signal:
scope: VFfuuVwYJ/VFfvAz07R
index: 1
started_at: 2026-04-02T15:28:04
---

# Create Turborepo Workspace And Shared Frontend Scripts

## Summary

Create the root Node workspace boundary for the frontend migration. This story establishes the top-level `package.json`, `turbo.json`, shared scripts, and verification wiring that later docs/runtime app slices depend on.

## Acceptance Criteria

- [ ] The repo defines a root Node workspace and Turborepo pipeline for frontend `build`, `lint`, and `test`. [SRS-01/AC-01] <!-- verify: automated, SRS-01:start:end -->
- [ ] `just quality` and `just test` invoke the shared frontend workspace entry points instead of per-folder ad hoc commands. [SRS-NFR-01/AC-02] <!-- verify: automated, SRS-NFR-01:start:end -->
