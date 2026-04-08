---
# system-managed
id: VGDNlYT3m
status: backlog
created_at: 2026-04-08T08:46:25
updated_at: 2026-04-08T08:50:22
# authored
title: Implement Self Discovering Workspace Entity Index And Cache
type: feat
operator-signal:
scope: VGDNcabks/VGDNgMbMW
index: 2
---

# Implement Self Discovering Workspace Entity Index And Cache

## Summary

Implement the self-discovering workspace entity inventory and cache so deterministic lookup runs against authored files, respects `.gitignore`, and can survive across turns without stale drift.

## Acceptance Criteria

- [ ] The resolver inventory is built from authored workspace files only and excludes ignored/generated paths through the shared workspace boundary policy. [SRS-02/AC-01] <!-- verify: cargo nextest run resolver_inventory_respects_workspace_boundary --no-tests pass, SRS-02:start:end -->
