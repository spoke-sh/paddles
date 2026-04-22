# VHUSCtMpx Review Notes

## Application Read-Model Ownership

- `src/application/read_model/` now owns `transcript`, `forensics`, `manifold`,
  and `projection`.
- `src/application/mod.rs` exports those projection types from the application
  boundary.
- `src/domain/model/mod.rs` no longer declares the projection modules directly;
  it only provides compatibility re-exports so existing internal callers can
  migrate without changing ownership back to the domain.
- `src/infrastructure/web/mod.rs` and
  `src/infrastructure/cli/interactive_tui.rs` now consume the application
  read-model types instead of relying on domain ownership for the moved
  surfaces.
