---
title: "feat(init): Scaffold foundational documents"
status: backlog
type: feat
---

## Description

The `keel init` command should be improved to scaffold the foundational documents for new projects.
Currently, it only creates the `.keel` directory and `keel.toml`.
To improve the onboarding experience, `keel init` should also create:
- `AGENTS.md`
- `INSTRUCTIONS.md`
- `CONSTITUTION.md`
- `ARCHITECTURE.md`
- `CONFIGURATION.md`
- `EVALUATIONS.md`
- `RELEASE.md`

This will ensure consistency across all `keel`-managed projects from the very beginning.

## Acceptance Criteria

- [ ] `keel init` creates the foundational documents in the current directory.
- [ ] The created documents are populated with the standard content from `keel`'s templates.
- [ ] The `keel doctor` command verifies the presence of these documents.
