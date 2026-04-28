# VOYAGE REPORT: Rename MechSuitService And Chambers To Idiomatic Modules

## Voyage Metadata
- **ID:** VI2sfbhqT
- **Epic:** VI2sJZcV9
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 5/5 stories complete

## Implementation Narrative
### Rename MechSuitService to AgentRuntime
- **ID:** VI2slwMGg
- **Status:** done

#### Summary
Mechanical, behavior-preserving rename of the `MechSuitService` god-object to `AgentRuntime`. Update every type reference, factory closure type, trait impl, test, and trace identifier across `src/`, `tests/`, `apps/`, and the keel artifacts that mention the type. No functional changes — public CLI flags, on-disk trace schemas, and HTTP routes remain identical.

#### Acceptance Criteria
- [x] `struct MechSuitService` is renamed to `struct AgentRuntime` and every `MechSuitService` reference across `src/` and `tests/` is updated (verified by `git grep -F MechSuitService` returning zero hits). [SRS-01/AC-01] <!-- verify: cargo test --lib, SRS-01:start:end -->
- [x] `cargo check`, `cargo test --lib`, and `cargo clippy --all-targets -- -D warnings` pass with the rename in place and no behavior change. [SRS-01/AC-02] <!-- verify: cargo test --lib, SRS-01:start:end -->
- [x] Public CLI flags, web HTTP routes, and persisted trace record schemas are unchanged — the rename is in-process types only. [SRS-01/AC-03] <!-- verify: cargo test --lib, SRS-01:start:end -->

### Rename Chamber Wrappers To Plain Modules
- **ID:** VI3BWyxR3
- **Status:** done

#### Summary
Delete the stateless `*Chamber` wrappers in `src/application/` (`RecursiveControlChamber`, `InterpretationChamber`, `SynthesisChamber`, `TurnOrchestrationChamber`, and any siblings) and migrate their methods to plain function modules. The new module names match SCOPE-02: `agent_loop`, `context_assembly`, `synthesis`, `turn`. Behavior unchanged; CLI flags, web routes, and trace schemas untouched.

#### Acceptance Criteria
- [x] Every `*Chamber` wrapper struct in `src/application/` is deleted (`InterpretationChamber`, `SynthesisChamber`, `ConversationReadModelChamber`, `RecursiveControlChamber`, `TurnOrchestrationChamber`); its methods are now free functions in modules named for the phase (`context_assembly`, `synthesis`, `conversation_read_model`, `agent_loop`, `turn`). `git grep -E '\\bChamber\\b' src/application/` returns only string-literal hits in test fixtures. [SRS-01/AC-01] <!-- verify: cargo test --lib, SRS-01:start:end -->
- [x] All call sites that did `self.service.foo_chamber().bar()` (or `self.foo_chamber().bar()` on `AgentRuntime`) are updated to call the module-level functions directly with `service` as the first argument; the corresponding `AgentRuntime::*_chamber()` accessors are removed. [SRS-01/AC-02] <!-- verify: cargo test --lib, SRS-01:start:end -->
- [x] `cargo check`, `cargo test --lib` (782 passing), and `cargo clippy --all-targets -- -D warnings` pass with the migration in place and no behavior change. [SRS-01/AC-03] <!-- verify: cargo test --lib, SRS-01:start:end -->

### Rename Recursive Control Module To Agent Loop
- **ID:** VI3BX17Sw
- **Status:** done

#### Summary
Rename `src/application/recursive_control.rs` to `src/application/agent_loop.rs`, update the `mod recursive_control;` declaration in `src/application/mod.rs` to `mod agent_loop;`, and update every `use` path. The renamed module reflects the industry-standard ReAct loop terminology. Behavior unchanged.

#### Acceptance Criteria
- [x] `src/application/recursive_control.rs` is renamed (via `git mv`) to `src/application/agent_loop.rs` and the `mod recursive_control;` declaration in `src/application/mod.rs` is now `mod agent_loop;`. [SRS-01/AC-01] <!-- verify: cargo test --lib, SRS-01:start:end -->
- [x] Every `use` path and accessor referencing `recursive_control` is updated to `agent_loop` (the `AgentRuntime::recursive_control()` accessor is renamed to `AgentRuntime::agent_loop()`); `git grep -E '\\brecursive_control\\b'` returns zero hits in `src/`. [SRS-01/AC-02] <!-- verify: cargo test --lib, SRS-01:start:end -->
- [x] `cargo check`, `cargo test --lib` (782 passing), and `cargo clippy --all-targets -- -D warnings` pass with the rename in place and no behavior change. [SRS-01/AC-03] <!-- verify: cargo test --lib, SRS-01:start:end -->

### Trust Operator Memory Over Probe Procedure
- **ID:** VI3fgYucO
- **Status:** done

#### Summary
Stop the harness from overriding what the operator wrote in AGENTS.md. Pass the full operator-memory documents alongside the summarized `InterpretationContext` into every action-selection `PlannerRequest`, render them in the planner system prompt as the **primary source of truth**, and reframe the controller-authored "Probe Required Local Tools" procedure as a **validating cache layer** that confirms operator-documented CLIs rather than prescribing generic `command -v <tool>` discovery sweeps. After this lands, "continue executing mission VI2q5DKHe" should produce `keel mission show VI2q5DKHe` as the first action even with a small local planner.

#### Acceptance Criteria
- [x] `PlannerRequest` carries `operator_memory: Vec<OperatorMemoryDocument>` populated at construction time from the OperatorMemory port; both production action-selection sites (initial and recursive) and the steering-review path call `.with_operator_memory(...)`. [SRS-01/AC-01] <!-- verify: cargo test --lib planner_request_carries_full_operator_memory_alongside, SRS-01:start:end -->
- [x] Sift and HTTP planner adapters render the full operator-memory documents in the planner prompt, ahead of the summarized interpretation context, with explicit precedence text declaring operator memory as the primary source of truth. [SRS-01/AC-02] <!-- verify: cargo test --lib planner_request_carries_full_operator_memory_alongside, SRS-01:start:end -->
- [x] The controller-authored procedure is renamed to "Validate And Cache Documented Local Tools" with a purpose that explicitly defers to operator memory; sample probe steps point at operator-documented tools (`command -v <tool-named-by-operator-memory>`) rather than prescribing a generic discovery sweep. [SRS-01/AC-03] <!-- verify: cargo test --lib validate_and_cache_procedure_does_not_prescribe, SRS-01:start:end -->

### Fix UTF-8 Boundary Panic In Truncate Helpers
- **ID:** VI3kNOiYB
- **Status:** done

#### Summary
Two private `truncate(s: &str, n: usize)` helpers — `src/infrastructure/adapters/http_provider.rs:3551` and `src/domain/model/read_model/projection.rs:283` — sliced strings by raw byte index (`&s[..n]`). When the cap landed inside a multi-byte UTF-8 character (e.g. the `─` U+2500 box-drawing character emitted by `keel health --scene`), the slice panicked with `byte index N is not a char boundary`. The user hit this in production immediately after the operator-memory trust change started routing real `keel` CLI output through the planner-bound summary. Fix both helpers to snap the cap down to the nearest `is_char_boundary` so the slice always lands on a valid UTF-8 boundary.

#### Acceptance Criteria
- [x] `truncate` in `src/infrastructure/adapters/http_provider.rs` snaps the byte cap down to the nearest `is_char_boundary` instead of slicing raw bytes; verified by a regression test that constructs the exact post-`keel-health` buffer that previously panicked at byte index 180. [SRS-01/AC-01] <!-- verify: cargo test --lib truncate_snaps_to_char_boundary_when_byte_cap_lands_inside_a_multibyte_char, SRS-01:start:end -->
- [x] `truncate` in `src/domain/model/read_model/projection.rs` receives the same fix so the projection layer cannot panic on box-drawing or other multi-byte tool output. [SRS-01/AC-02] <!-- verify: cargo test --lib truncate_returns_full_string_when_under_cap, SRS-01:start:end -->
- [x] `cargo test --lib` and `cargo clippy --all-targets -- -D warnings` pass; full suite count rises to 782. [SRS-01/AC-03] <!-- verify: cargo test --lib, SRS-01:start:end -->


