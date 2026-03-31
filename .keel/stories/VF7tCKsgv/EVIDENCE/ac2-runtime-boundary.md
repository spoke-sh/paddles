# AC2 Proof: legacy-engine Removed From Core Runtime Boundary

- `Cargo.toml` contains no `legacy-core`, `legacy-provider`, or `legacy-tools` runtime dependencies.
- `rg -n "legacy-engine" Cargo.toml src` returns no matches, confirming the core runtime modules no longer import legacy-engine types.
- `src/application/mod.rs`, `src/main.rs`, and `src/domain/ports/mod.rs` now depend only on local ports and the Sift-backed adapter for prompt execution.
