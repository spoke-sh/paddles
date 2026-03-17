# Keel Policy: The Executable Invariants

> The operational invariants and constraints that govern the Keel Simulator.

## 1. The Core Objective: Zero Drift
The primary goal of the engine is to eliminate **Drift**. Progress is blocked if any of the following are detected by the `doctor`:
- **Structural Drift:** Missing files, invalid IDs, or broken frontmatter.
- **Architectural Drift:** Working in a bounded context with a `proposed` ADR.
- **Requirement Drift:** Stories missing SRS references or acceptance criteria.
- **Scaffold Drift:** Presence of placeholder text (e.g., `{{goal}}`, `Item 1`).

## 2. Entity Invariants
Every entity in the `.keel/` directory must adhere to these structural rules:
- **Missions:** Must have a `CHARTER.md` with at least one `board:`-verifiable goal to be `active`.
- **Epics:** Status is *derived* from voyages. An epic is `draft` until its first voyage is `planned`.
- **Voyages:** Must have an `SRS.md` and `SDD.md` with authored content to transition from `draft` to `planned`.
- **Stories:** Must link to a `voyage/SRS` requirement via the `[SRS-XX/AC-YY]` format.

## 3. Mission Completion Policy
A mission is considered **Achieved** only when:
- **Goals Met:** All `board:`-verifiable goals in the `CHARTER.md` are terminal and satisfied in the board state.
- **No Open Work:** All child entities (Stories, Voyages, Epics, Bearings, ADRs) are in terminal states.
- **Documented:** At least one entry exists in `LOG.md` to summarize the session.

## 4. The Transition State Machine
State transitions are explicit and gated. An actor cannot "force" a state without satisfying the prerequisites:
- **Thaw:** Requires valid SRS references and authored acceptance criteria.
- **Start:** Requires the parent Voyage to be `active`.
- **Submit:** Requires all acceptance criteria to have recorded verification proofs.
- **Accept:** Requires human verification of the submitted evidence.

## 5. Priority Ranking (The `calculate_next` Logic)
The engine resolves the "next move" using a strict priority hierarchy:
1.  **Diagnostics (Priority 0):** Critical board health issues must be fixed first.
2.  **Architectural Decisions:** Proposed ADRs block all implementation in their context.
3.  **Human Acceptance:** Piling up "Needs Verification" items blocks the flow.
4.  **Strategic Planning:** Draft voyages and epics must be planned/decomposed.
5.  **Active Work:** Continuing in-progress stories takes precedence over starting new ones.
6.  **Research:** Exploration of bearings happens when the implementation queue is clear.

## 6. The Compact Status Contract
When querying with `--status`, the engine MUST:
- Return exactly **three** bullets.
- Prioritize high-signal "blockers" over "routine work."
- Use **action-oriented** language (e.g., "Review ADR," "Start Story").
- Style IDs and Titles consistently to minimize cognitive load.

## 7. Mission Archetypes
To prevent "Strategic Fog," every mission must align with one of four archetypes:
- **Strategic (Foundation):** Large-scale value shifts or architectural foundations.
- **Maintenance (Healing):** Purely focused on reaching/maintaining Zero Drift.
- **Exploratory (Discovery):** Dominated by `Bearings` and `Play`; focuses on reducing the fog of war.
- **Bridging (Realization):** Explicitly graduates conclusive research (Graduated Bearings) into `Planned` implementation work.
