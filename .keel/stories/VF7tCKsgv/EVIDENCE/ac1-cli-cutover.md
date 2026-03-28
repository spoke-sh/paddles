# AC1 Proof: CLI Flows Remain Operational

- `cargo run -- --help` succeeds after the cutover and renders the full CLI surface, including `--prompt`, `--model`, `--verbose`, and the interactive-mode inputs.
- `src/main.rs` preserves both execution branches:
  - the single-prompt branch prints `Chord Response` after `service.process_prompt(&prompt, paths).await`
  - the interactive branch loops on stdin, reuses `paths.clone()`, and prints `Chord Response` for each turn
- Both branches now route through the same `MechSuitService::process_prompt` entrypoint, so the cutover did not split CLI behavior across separate runtimes.
