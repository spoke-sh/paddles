# Convert Web Surfaces To Turborepo React Workspace - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Establish a single Turborepo-managed Node/React frontend workspace that owns both the public docs site and the runtime web application shell. | board: VFfuuVwYJ |
| MG-02 | Introduce a tested React runtime web app that can progressively absorb the current Rust-embedded web shell without cutting operators off from existing routes or backend APIs. | board: VFfuuVwYJ |
| MG-03 | Keep the migration honest: documentation, build scripts, and verification loops must describe the staged cutover clearly instead of claiming full React ownership before parity exists. | manual: README.md, ARCHITECTURE.md, and CONFIGURATION.md describe the staged migration and current source of truth accurately |

## Constraints

- Preserve the local-first backend and existing Axum API surface; this mission does not replace the Rust service with a remote frontend backend.
- Migrate in sealed slices. The embedded HTML shell may remain temporarily, but each slice must reduce the gap to React rather than adding a parallel dead-end UI.
- Keep docs and runtime web apps inside one Turborepo workspace so quality, test, and E2E verification run through the same Node entry points.
- Add real tests at the workspace, app, and browser levels. Do not claim the React app is production-ready without unit/integration/E2E coverage.
- Avoid introducing hosted frontend infrastructure, CDNs, or cloud-only build requirements.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when epic VFfuuVwYJ is verified and only optional polish or long-tail parity work remains outside the planned slices
- YIELD to human when product decisions are needed about route ownership cutover, React app visual redesign beyond parity, or backend/frontend deployment topology changes
