# Interpretation Context TUI Visibility - Product Requirements

## Problem Statement

The TUI shows only a generic summary and source list for the interpretation context, hiding the structured categories (documents, tool hints, procedures, precedence) that paddles actually derived — users cannot validate what paddles understood from their guidance.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Users can see what paddles derived from their guidance at a glance | TUI event shows category counts and key details, not just a summary string | Visible in default verbosity for slow steps, full detail at -v |
| GOAL-02 | Users can drill into the full interpretation at higher verbosity | -vv shows document excerpts, tool hints, and decision procedures inline | All structured fields rendered |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Interactive operator | Developer using paddles interactively | Validate that paddles understood their guidance correctly before it makes decisions |

## Scope

### In Scope

- [SCOPE-01] Richer TurnEvent::InterpretationContext payload with structured category counts
- [SCOPE-02] Tiered TUI rendering: compact at default, detailed at -v, full at -vv
- [SCOPE-03] Emit guidance graph expansion as a separate visible event

### Out of Scope

- [SCOPE-04] Modifying the interpretation derivation logic itself (that's the quality epic)
- [SCOPE-05] Web UI rendering of interpretation context

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | TurnEvent::InterpretationContext carries category counts (documents, hints, procedures) alongside summary | GOAL-01 | must | Enables compact rendering without losing structure |
| FR-02 | Default TUI renders interpretation event with category breakdown (e.g. "2 docs, 3 hints, 1 procedure") | GOAL-01 | must | Quick validation at a glance |
| FR-03 | At -v, TUI shows document sources with excerpt previews and tool hint summaries | GOAL-02 | must | Drill-down without full verbosity |
| FR-04 | At -vv, TUI shows full rendered interpretation including procedure steps | GOAL-02 | should | Complete transparency for debugging |
| FR-05 | Emit a separate TurnEvent for guidance graph expansion showing docs discovered and depth reached | GOAL-01 | should | Makes the graph walk visible |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Rendering must respect existing verbosity tier and pace promotion system | GOAL-01 | must | Consistent UX |
| NFR-02 | No additional model calls for rendering — use what's already derived | GOAL-01 | must | No cost increase |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Compact rendering | Manual: run at default verbosity, confirm category counts visible | Session output |
| Detailed rendering | Manual: run at -v and -vv, confirm progressive detail | Session output |
| Graph expansion event | Manual: confirm separate event shows discovered documents | Session output |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing InterpretationContext struct has enough data for rendering | May need to carry additional fields | Review struct during implementation |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Exact compact format for category counts may need UX iteration | Operator | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Default verbosity shows "2 docs, 3 hints, 1 procedure" style breakdown
- [ ] -v shows source names with excerpt previews
- [ ] -vv shows full interpretation context render
<!-- END SUCCESS_CRITERIA -->
