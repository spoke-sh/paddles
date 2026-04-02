---
# system-managed
id: VFfvB0g8R
status: done
created_at: 2026-04-02T15:25:52
updated_at: 2026-04-02T15:55:28
# authored
title: Move The Docs App Into The Shared Frontend Workspace
type: feat
operator-signal:
scope: VFfuuVwYJ/VFfvAz07R
index: 4
started_at: 2026-04-02T15:51:17
completed_at: 2026-04-02T15:55:28
---

# Move The Docs App Into The Shared Frontend Workspace

## Summary

Move the existing Docusaurus documentation site into the shared frontend workspace and keep its existing verification surface intact.

## Acceptance Criteria

 - [x] The docs app lives under the shared frontend workspace without losing typecheck, build, or browser verification. [SRS-02/AC-01] <!-- verify: nix develop --command sh -lc 'cargo test -q infrastructure::dev_workflow_contracts::docs_app_defines_browser_e2e_verification && just quality && just test', SRS-02:start:end, proof: ac-1.log-->
