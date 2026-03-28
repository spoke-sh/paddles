# AC1 Proof: Runtime Cutover

- `cargo test` passed after the cutover, including the `sift_agent` unit suite.
- `just quality` passed (`cargo fmt --all --check` and `cargo clippy --all-targets --all-features -- -D warnings`).
- `src/application/mod.rs` now constructs and stores `SiftAgentAdapter` as the active runtime session controller.
- `src/main.rs` no longer constructs `wonopcode_core::Instance`; prompt execution now flows through `MechSuitService` and the Sift adapter.
- `Cargo.toml` removes `wonopcode-core`, `wonopcode-provider`, and `wonopcode-tools` from runtime dependencies.
