# AGENTS.md

Shared guidance for AI agents using `paddles` to work on `paddles`.

## Role

You are operating inside the `paddles` repository. Your job is to advance the
codebase and the `.keel/` board while keeping the repo, worktree, and board in
a healthy state.

This document is an operator contract, not a runtime specification. Use it to
orient yourself in the environment. For program behavior, architecture, and
policy, defer to the documents listed below.

## Core Principles

1. **Garden First**: Resolve board drift, doctor errors, broken tests, and open
   loops before escalating to the human. A healthy garden supports healthy work.
2. **Work In Sealed Slices**: Prefer small, coherent changes that land with a
   single sealing commit. Each slice is self-contained and verifiable.
3. **Protect Local-First Constraints**: The local-first runtime is the
   foundation — preserve it and justify any new network dependencies via ADR.
4. **Respect Existing Work**: Work with the current tree. Preserve unrelated
   user changes unless the user explicitly asks otherwise.
5. **Use The Board Engine**: Manage project state through the `keel` CLI. The
   board engine maintains structural integrity of `.keel` artifacts.
6. **Escalate Only For Real Decisions**: Handle tactical implementation and drift
   autonomously. Ask the human when product direction, UX behavior, or design
   tradeoffs need input.
7. **Update The Right Source Of Truth**: When behavior changes, update the doc
   that owns that behavior. Each contract lives in exactly one place.

## Canonical Turn Loop

Keel's operator rhythm in this repo is:

- **Orient**: Run `keel health --scene`, `keel flow --scene`, and
  `keel doctor`.
- **Inspect**: Run `keel mission next`, `keel pulse`, and
  `keel workshop`. Use `keel screen --static` or `keel topology --static` when
  board geometry is unclear.
- **Pull**: Select one explicit slice with `keel next --role <role>` or follow
  the mission/story the human named.
- **Ship**: Implement the slice, record proof, and advance lifecycle state.
- **Close**:
  - Run `git status` before the commit boundary when you need an open-loop
    check.
  - Land the sealing commit with `git commit`. Installed hooks run repo checks
    and append `keel doctor --status` output to the commit message.
  - If the commit fails, fix the reported issue and retry instead of leaving the
    loop partially open.
- **Re-orient**: After the commit lands, run `keel doctor` and
  `keel flow --scene` to see what the board needs next.

## Session Start

When a human opens the chat or explicitly pokes the system, immediately perform
the orient/inspect half of the loop:

1. Run `keel poke "Human interaction in chat"`.
2. Run `keel health --scene`.
3. Run `keel mission next`, `keel pulse`, and `keel workshop`.
4. Run `keel flow --scene`.
5. Run `keel doctor`.

## Working Rules

- Use the raw `keel` CLI directly.
- Treat `keel doctor` as the source of truth for board integrity.
- If board-mutating lifecycle commands produce `.keel` churn, include that churn
  in the same sealing commit.
- Re-run orientation after each sealing commit instead of stopping at “done.”
- Keep docs synchronized with behavior changes, but update the owning docs:
  `README.md`, `POLICY.md`, `ARCHITECTURE.md`, `CONFIGURATION.md`,
  `INSTRUCTIONS.md`, or planning artifacts as appropriate.

## Source Documents

Read these as needed:

- [INSTRUCTIONS.md](INSTRUCTIONS.md) for the full procedural loops and checklists
- [README.md](README.md) for repository navigation and high-level architecture
- [CONSTITUTION.md](CONSTITUTION.md) for collaboration philosophy and bounds
- [POLICY.md](POLICY.md) for runtime and operational invariants
- [ARCHITECTURE.md](ARCHITECTURE.md) for implementation boundaries and current architecture
- [CONFIGURATION.md](CONFIGURATION.md) for concrete runtime topology and lane configuration
- [.keel/adrs/](.keel/adrs/) for binding architectural decisions

## Decision Resolution Hierarchy

When faced with ambiguity, resolve decisions in this descending order:

1. **ADRs** in `.keel/adrs/`
2. **CONSTITUTION**
3. **POLICY**
4. **ARCHITECTURE**
5. **Current planning artifacts** such as PRD/SRS/SDD for the active mission
