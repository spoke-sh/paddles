# VHUSCJwQG Review Notes

## Chamber Ownership

- `src/application/mod.rs` now exposes chamber accessors and delegates the
  public prompt-processing and replay/projection API through
  `TurnOrchestrationChamber` and `ConversationReadModelChamber`.
- `src/application/turn_orchestration.rs` owns the turn-processing and
  thread-candidate orchestration flow that previously lived inline on
  `MechSuitService`.
- `src/application/conversation_read_model.rs` owns the replay and projection
  boundary used by transcript, forensics, manifold, delegation, and shared
  projection snapshots.
- `src/application/synthesis_chamber.rs` owns recent-turn recovery,
  specialist-runtime note assembly, and completion finalization.
- `src/application/interpretation_chamber.rs` and
  `src/application/recursive_control.rs` provide explicit seams for
  interpretation and recursive loop ownership, so orchestration code now
  composes chamber services instead of implementing every concern inline on the
  top-level application service.
