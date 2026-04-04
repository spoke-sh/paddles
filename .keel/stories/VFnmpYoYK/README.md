---
# system-managed
id: VFnmpYoYK
status: done
created_at: 2026-04-03T23:42:53
updated_at: 2026-04-04T00:15:27
# authored
title: Render Applied Edit Diffs In The Web Runtime Stream
type: feat
operator-signal:
scope: VFnmIbFW2/VFnmfzD3E
index: 2
started_at: 2026-04-04T00:15:12
completed_at: 2026-04-04T00:15:27
---

# Render Applied Edit Diffs In The Web Runtime Stream

## Summary

Render the shared applied-edit artifact in the web runtime stream so operators can see which file changed and inspect the diff inline instead of inferring edits from generic tool chatter.

## Acceptance Criteria

- [x] The web runtime stream renders applied-edit artifacts with file identity and diff hunks using the shared runtime contract [SRS-03/AC-01] <!-- verify: npm --workspace @paddles/web exec vitest run src/runtime-app.test.tsx, SRS-03:start:end, proof: ac-1.log-->
