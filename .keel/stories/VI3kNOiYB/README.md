---
# system-managed
id: VI3kNOiYB
status: done
created_at: 2026-04-27T22:11:16
updated_at: 2026-04-27T22:13:50
# authored
title: Fix UTF-8 Boundary Panic In Truncate Helpers
type: fix
operator-signal:
scope: VI2sJZcV9/VI2sfbhqT
index: 5
started_at: 2026-04-27T22:13:19
completed_at: 2026-04-27T22:13:50
---

# Fix UTF-8 Boundary Panic In Truncate Helpers

## Summary

Two private `truncate(s: &str, n: usize)` helpers — `src/infrastructure/adapters/http_provider.rs:3551` and `src/domain/model/read_model/projection.rs:283` — sliced strings by raw byte index (`&s[..n]`). When the cap landed inside a multi-byte UTF-8 character (e.g. the `─` U+2500 box-drawing character emitted by `keel health --scene`), the slice panicked with `byte index N is not a char boundary`. The user hit this in production immediately after the operator-memory trust change started routing real `keel` CLI output through the planner-bound summary. Fix both helpers to snap the cap down to the nearest `is_char_boundary` so the slice always lands on a valid UTF-8 boundary.

## Acceptance Criteria

- [x] `truncate` in `src/infrastructure/adapters/http_provider.rs` snaps the byte cap down to the nearest `is_char_boundary` instead of slicing raw bytes; verified by a regression test that constructs the exact post-`keel-health` buffer that previously panicked at byte index 180. [SRS-01/AC-01] <!-- verify: cargo test --lib truncate_snaps_to_char_boundary_when_byte_cap_lands_inside_a_multibyte_char, SRS-01:start:end -->
- [x] `truncate` in `src/domain/model/read_model/projection.rs` receives the same fix so the projection layer cannot panic on box-drawing or other multi-byte tool output. [SRS-01/AC-02] <!-- verify: cargo test --lib truncate_returns_full_string_when_under_cap, SRS-01:start:end -->
- [x] `cargo test --lib` and `cargo clippy --all-targets -- -D warnings` pass; full suite count rises to 782. [SRS-01/AC-03] <!-- verify: cargo test --lib, SRS-01:start:end -->
