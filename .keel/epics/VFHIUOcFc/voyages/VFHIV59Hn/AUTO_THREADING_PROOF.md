# Auto-Threading Proof

## Scope

Mission `VFHITRYFP` delivers model-driven auto-threading for interactive
sessions without turning `transit-core` into the conversation API boundary.

## Delivered Behaviors

1. Interactive sessions now keep one durable conversation root task and reuse it
   across turns through [session.rs](/home/alex/workspace/spoke-sh/paddles/src/application/session.rs).
2. Steering prompts captured during an active turn are no longer opaque queue
   strings. They become typed `ThreadCandidate` values in
   [threading.rs](/home/alex/workspace/spoke-sh/paddles/src/domain/model/threading.rs).
3. Thread classification is model-directed through the new constrained
   `select_thread_decision` planner path in
   [planning.rs](/home/alex/workspace/spoke-sh/paddles/src/domain/ports/planning.rs),
   [sift_planner.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/sift_planner.rs),
   and [sift_agent.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/sift_agent.rs).
4. Split and merge outcomes are explicit trace records, not UI-only state, in
   [traces.rs](/home/alex/workspace/spoke-sh/paddles/src/domain/model/traces.rs) and
   [mod.rs](/home/alex/workspace/spoke-sh/paddles/src/application/mod.rs).
5. The default transcript surfaces steering capture, thread decisions, active
   thread identity, and merge-back outcomes in
   [interactive_tui.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/cli/interactive_tui.rs).
6. Embedded transit replay stays generic while branch metadata now reuses
   upstream branch helper labels in
   [trace_recorders.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/trace_recorders.rs).

## Verification

```text
cargo test -q
just quality
cargo nextest run
```

## Current Limits

- Auto-threading is checkpoint-bounded. A steering prompt is captured while the
  active turn runs, classified after the current turn reaches a safe boundary,
  and then executed on the selected thread.
- `paddles` still runs one local generation path at a time. Child threads are
  durable and replayable, but not truly concurrent local model sessions yet.
- Merge-back is explicit lineage plus recorded outcome. It is not hidden history
  rewrite.
