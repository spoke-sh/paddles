# Release Process

`paddles` follows a structured release process to ensure high-fidelity delivery of agentic capabilities.

## Versioning

`paddles` adheres to [Semantic Versioning 2.0.0](https://semver.org/).

## Release Readiness Checklist

Before performing a release, the following invariants must be satisfied:

1.  **Zero Drift:** `keel doctor` must report 100% board integrity.
2.  **Verified State:** All mission goals for the release must be in the `Verified` state.
3.  **Quality Gate:** `just quality` (formatting and linting) must pass 100%.
4.  **Verification Gate:** `just test` must pass 100% in the current environment.
5.  **Documentation:** `CHANGELOG.md` must be updated with the session digest and key tactical moves.

## How to Perform a Release

### 1. Stabilization
Ensure the pacemaker is stable and all recent moves are committed.
```bash
keel poke "Stabilizing for release vX.Y.Z"
git add .
git commit -m "chore(release): prepare for vX.Y.Z"
```

### 2. Version Bump
Update the version in `Cargo.toml`.

### 3. Tagging
Create a signed git tag for the release.
```bash
git tag -s vX.Y.Z -m "Release vX.Y.Z"
```

### 4. Artifact Generation
Build the release artifact to verify compilation.
```bash
cargo build --release
```

---

*"Release is the moment the simulation becomes reality."*
