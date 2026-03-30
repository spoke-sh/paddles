---
# system-managed
id: VFNcuA8df
status: backlog
created_at: 2026-03-30T12:20:23
updated_at: 2026-03-30T12:24:40
# authored
title: Cache Directory Persistence And Boot Loading
type: feat
operator-signal:
scope: VFNccFj7d/VFNcoxjU3
index: 2
---

# Cache Directory Persistence And Boot Loading

## Summary

Serialize and deserialize the reservoir to `~/.cache/paddles/step_timing.json`. Load at application boot (missing/corrupt file → empty reservoir). Flush after each turn completes.

Key decisions:
- serde_json for serialization (already a dependency)
- Cache directory created on first write with mkdir_all
- Corrupt or unreadable file silently starts fresh (cache, not config)
- Flush triggered from the TUI after TurnFinished, not from the model thread

## Acceptance Criteria

- [ ] Reservoir round-trips through JSON serialization [SRS-05/AC-01] <!-- verify: test, SRS-05:start:end -->
- [ ] Missing file produces an empty reservoir on load [SRS-07/AC-02] <!-- verify: test, SRS-07:start:end -->
- [ ] Corrupt file produces an empty reservoir on load [SRS-07/AC-03] <!-- verify: test, SRS-07:start:end -->
- [ ] Cache directory is created if it does not exist [SRS-06/AC-04] <!-- verify: test, SRS-06:start:end -->
