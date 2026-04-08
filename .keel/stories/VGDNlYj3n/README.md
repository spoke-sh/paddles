---
# system-managed
id: VGDNlYj3n
status: done
created_at: 2026-04-08T08:46:25
updated_at: 2026-04-08T09:15:52
# authored
title: Resolve Symbols And Fuzzy Path Hints Into Authored Files
type: feat
operator-signal:
scope: VGDNcabks/VGDNgMbMW
index: 3
started_at: 2026-04-08T09:13:49
completed_at: 2026-04-08T09:15:52
---

# Resolve Symbols And Fuzzy Path Hints Into Authored Files

## Summary

Resolve concrete path hints, basename/component names, and symbol-like fragments into authored workspace file candidates with deterministic ranking and explicit ambiguity reporting.

## Acceptance Criteria

- [x] Exact relative paths, basename/component hints, and symbol-like path fragments resolve through one deterministic resolver path without IDE or LSP dependencies. [SRS-03/AC-01] <!-- verify: cargo nextest run resolver_supports_exact_path_basename_and_symbol_hints --no-tests pass, SRS-03:start:end, proof: ac-1.log-->
