---
# system-managed
id: VJXfKtukt
status: backlog
created_at: 2026-05-13T15:28:17
updated_at: 2026-05-13T15:29:36
# authored
title: Migrate Planner Prompts To Shared Schema
type: feat
operator-signal:
scope: VJXeteRQ5/VJXf4hlYW
index: 1
---

# Migrate Planner Prompts To Shared Schema

## Summary

Migrate Sift/local and HTTP/remote planner prompt builders so action-schema
text comes from the shared renderer. Provider-specific transport instructions
remain adapter-local.

## Acceptance Criteria

- [ ] Sift initial, recursive, retry, and redecision prompts consume the shared schema renderer. [SRS-01/AC-01] <!-- verify: test, SRS-01:start:end -->
- [ ] HTTP planner prompts consume the shared schema renderer while preserving native-tool, structured JSON, and prompt-envelope transport instructions. [SRS-02/AC-02] <!-- verify: test, SRS-02:start:end -->
- [ ] `rg` or equivalent proof shows no remaining adapter-local planner action JSON example lists outside the shared renderer. [SRS-03/AC-03] <!-- verify: command, SRS-03:start:end -->
