# AC2 Proof: wonopcode Removed From Core Runtime Boundary

- `Cargo.toml` contains no `wonopcode-core`, `wonopcode-provider`, or `wonopcode-tools` runtime dependencies.
- `rg -n "wonopcode" Cargo.toml src` returns no matches, confirming the core runtime modules no longer import wonopcode types.
- `src/application/mod.rs`, `src/main.rs`, and `src/domain/ports/mod.rs` now depend only on local ports and the Sift-backed adapter for prompt execution.
