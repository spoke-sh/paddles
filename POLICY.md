# Paddles Policy: Operational Invariants

> The operational invariants and constraints that govern the Paddles Mech Suit.

## 1. Boot Invariants (The Clean Boot)
A boot sequence is only valid (CLEAN) if:
- **Credits**: Initial balance is non-negative (default 0).
- **Weights**: Calibration values are within the `min_weight` and `max_weight` defined in the `Constitution`.
- **Dogma**: The session does not trigger `reality_mode` or equivalent dogma violations.

## 2. Zero Drift Policy
Progress is blocked if structural or requirement drift is detected:
- **Requirement Drift**: Stories missing SRS/AC links or verification proofs.
- **Structural Drift**: Broken board integrity or orphaned entities.
- **Scaffold Drift**: Unfilled template placeholders (`{{goal}}`).

## 3. Local Capacity Invariant
Features should be implemented using local inference (`candle`) or local-first tools whenever possible. New network dependencies must be justified via an ADR.

## 4. Entity State Machine
Follow strict transition gates for all `.keel/` entities:
- **Planned**: Requires SRS/SDD authored content.
- **Started**: Requires an active parent Voyage.
- **Submitted**: Requires recorded proof for every Acceptance Criterion.
- **Verified**: Requires human sign-off of the submitted evidence.

## 5. Pacemaker Synchronization
Every commit that modifies the board MUST be pace-set by `keel poke` and include the `keel doctor --status` Importance Snapshot in the commit message.

## 6. Mission Achievement
A mission is **Achieved** only when:
- **Goals Met**: All `board:` goals are satisfied by terminal child entities.
- **Work Closed**: No open implementation work remains in the mission's scope.
- **Log Sealed**: A final session digest is recorded in `LOG.md`.
