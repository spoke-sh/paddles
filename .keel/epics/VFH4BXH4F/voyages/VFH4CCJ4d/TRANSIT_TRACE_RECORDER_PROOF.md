# Transit-Aligned Trace Recorder Proof

## Scope

Mission `VFH489exY` adds a paddles-owned trace contract, a recorder boundary
separate from transcript rendering, artifact envelopes for large turn payloads,
and an embedded `transit-core` recorder adapter with replay/checkpoint proof.

## Delivered Contract

- `src/domain/model/traces.rs`
  Defines stable task, turn, record, branch, artifact, and checkpoint ids plus
  the `TraceRecord` contract.
- `src/domain/ports/trace_recording.rs`
  Defines the storage-neutral `TraceRecorder` port and the noop baseline.
- `src/infrastructure/adapters/trace_recorders.rs`
  Implements in-memory recording and the embedded `transit-core` adapter.
- `src/application/mod.rs`
  Projects live runtime transitions into durable trace records while preserving
  the existing `TurnEventSink` UI path.

## Runtime Proof

The application-level test `process_prompt_records_trace_contract_records_beside_turn_events`
proves that a normal `MechSuitService` turn now records:

- a task root record
- planner action selection
- a completion checkpoint

without replacing or depending on the renderer-only event sink.

## Embedded Transit Proof

The adapter-level test `transit_recorder_replays_root_and_verifies_checkpoint`
proves that the embedded adapter can:

- create a local root stream
- append paddles trace records
- create and verify a lineage checkpoint
- replay the recorded trace back into `paddles` `TraceRecord` values

No networked `transit` server is required.

## Validation

Executed during this voyage:

- `cargo test -q`
- `just quality`
- `cargo test process_prompt_records_trace_contract_records_beside_turn_events`
- `cargo test transit_recorder_replays_root_and_verifies_checkpoint`

Note: `transit-core` currently emits an upstream deprecation warning from
`object_store::path::Path::child`, but the recorder proof and quality gate pass.
