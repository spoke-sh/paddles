# Upgrade Web UI Node Dependencies - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Remove npm audit vulnerabilities from the web UI workspace dependency graph without regressing the docs or web UI build/test workflow. | board: VHvKkR50r |

## Constraints

- Keep the change local to npm workspace manifests, lockfile, and necessary board artifacts.
- Prefer non-breaking dependency updates; only accept semver-major movement when tests prove compatibility.

## Halting Rules

- Do not halt while `npm audit` still reports vulnerabilities after dependency changes.
- Do not halt while docs or web UI npm quality gates are failing.
- Halt after the scoped story is accepted, the mission goal is achieved, and the dependency refresh is committed.
