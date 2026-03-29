# Define Transit-Aligned Trace Recorder Boundary - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-03-29T09:58:30-07:00

- Delivered a paddles-owned trace contract with stable task, turn, record,
  branch, artifact, and checkpoint identifiers.
- Split durable turn recording from transcript rendering through a dedicated
  `TraceRecorder` port and runtime trace projection.
- Added noop, in-memory, and embedded `transit-core` recorders plus replay and
  checkpoint verification proof.
- Updated foundational docs to document the recorder boundary, artifact
  envelopes, and the embedded-vs-server distinction honestly.

## 2026-03-29T09:57:55

Mission achieved by local system user 'alex'
