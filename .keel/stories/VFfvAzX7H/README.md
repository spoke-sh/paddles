---
# system-managed
id: VFfvAzX7H
status: done
created_at: 2026-04-02T15:25:52
updated_at: 2026-04-02T15:47:03
# authored
title: Create Turborepo Workspace And Shared Frontend Scripts
type: feat
operator-signal:
scope: VFfuuVwYJ/VFfvAz07R
index: 1
started_at: 2026-04-02T15:28:04
completed_at: 2026-04-02T15:47:03
---

# Create Turborepo Workspace And Shared Frontend Scripts

## Summary

Create the root Node workspace boundary for the frontend migration. This story establishes the top-level `package.json`, `turbo.json`, shared scripts, and verification wiring that later docs/runtime app slices depend on.

## Acceptance Criteria

- [x] The repo defines a root Node workspace and Turborepo pipeline for frontend `build`, `lint`, and `test`. [SRS-01/AC-01] <!-- verify: sh -lc 'cargo test -q infrastructure::dev_workflow_contracts::root_workspace_package_defines_shared_scripts_and_workspaces && cargo test -q infrastructure::dev_workflow_contracts::turbo_config_exists_for_frontend_workspace && cargo test -q infrastructure::dev_workflow_contracts::frontend_apps_exist_under_apps_directory', SRS-01:start:end, proof: ac-1.log-->
- [x] `just quality` and `just test` invoke the shared frontend workspace entry points instead of per-folder ad hoc commands. [SRS-NFR-01/AC-02] <!-- verify: nix develop --command sh -lc 'just quality && just test', SRS-NFR-01:start:end, proof: ac-2.log-->
