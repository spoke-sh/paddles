# Sift-Native Tool Runtime - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-03-27T19:45:00Z

- Created mission `VF7t3HtQY` to cut Paddles over from wonopcode-owned runtime orchestration to a Sift-native controller with retained context and local tools.
- Decomposed the mission into epic `VF7t633ux`, voyage `VF7tAvs7B`, and three execution stories covering controller cutover, local tool surface, and dependency/documentation cleanup.

## 2026-03-28T03:28:03Z

- Replaced the wonopcode prompt loop in `MechSuitService` with the new `SiftAgentAdapter` session controller and removed wonopcode runtime dependencies from the application entry path.
- Hardened the local tool surface by rejecting symlink escapes and surfacing non-zero `shell` and `apply_patch` exits as recoverable tool failures, then added regression coverage for both cases.
- Recorded proof for story `VF7tCKEgw` and submitted it for human verification.

## 2026-03-28T06:10:57Z

- Accepted story `VF7tCKEgw` as manager after review of the attached proof logs, clearing the workshop bench for the next delivery slice.

## 2026-03-28T06:12:11Z

- Started story `VF7tCKUgx`, attached proof for the local tool surface and searchable tool-output context, and submitted it for manual verification.

## 2026-03-28T06:12:40Z

- Accepted story `VF7tCKUgx` as manager after review of the attached proof logs, unblocking the final runtime/docs cutover slice.

## 2026-03-28T06:14:04Z

- Started story `VF7tCKsgv`, attached proof for CLI continuity, runtime-boundary cleanup, and verbose debugging output, and submitted it for manual verification.
