# Keel Constitution: The Simulator Philosophy

> Minimize drift through planning, execution, and verification.

This document captures the foundational principles of how Keel operates as an **Agentic SDLC Simulator**: the human-agent collaboration model, the resolution hierarchy, and the formal boundaries that keep autonomous delivery aligned.

## The Resolution Hierarchy

Decisions in the simulator flow through a hierarchy of resolution, each level defining the physics for the next:

1. **ADRs** define the binding architectural physics.
2. **CONSTITUTION** defines the collaboration philosophy and intent.
3. **FORMAL RULES** define the executable engine invariants.
4. **ARCHITECTURE** defines the physical source layout and boundaries.
5. **PLANNING** (PRD/SRS/SDD) defines the scoped mission work.

## Core Belief

**Humans author the physics. Agents execute within the simulation. Verification confirms the state.**

The goal is not to remove humans from software development, but to place human judgment where it matters most: defining architectural constraints, strategic direction, and accepting the final state. Everything else is delegated to agents operating within these high-fidelity rules.

## The Collaboration Model

Keel operates as a **2-queue pull system** coordinating work between humans and agents:

```
┌──────────────────────────────────────────────────────────────────────────┐
│                        Human-Agent Collaboration                         │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   HUMAN RESPONSIBILITIES                  AGENT RESPONSIBILITIES         │
│   ┌────────────────────────────┐         ┌──────────────────────────┐    │
│   │ • Architectural decisions  │         │ • Implementation         │    │
│   │ • Scope approval           │         │ • Testing                │    │
│   │ • Acceptance/rejection     │         │ • Verification execution │    │
│   │ • Research direction       │         │ • Documentation          │    │
│   └────────────────────────────┘         └──────────────────────────┘    │
│              │                                       │                   │
│              └─────────────── ADRs ──────────────────┘                   │
│                          (the boundary)                                  │
└──────────────────────────────────────────────────────────────────────────┘
```

Each actor pulls work when ready. No push coordination needed. Clear handoffs at well-defined boundaries.

## The Decision Hierarchy

Decisions flow through a hierarchy, each level constraining the next:

```
ADRs (Architecture Decision Records)     ← Human decides
    ↓ constrains
Epics (PRD)                              ← Human approves
    ↓ constrains
Voyages (SRS + SDD)                      ← Human approves
    ↓ constrains
Stories                                  ← Agent executes
    ↓ verified by
Automated Verification                   ← Machine confirms
    ↓ sealed by
Human Acceptance                         ← Human closes the loop
```

### ADRs: The Constitutional Layer

Architecture Decision Records live at `.keel/adrs/` and form the **constitutional layer** of the system. They capture:

- **Context** — Why this decision is needed
- **Decision** — What we decided
- **Constraints** — What agents MUST/MUST NOT do
- **Verification** — How compliance is checked

ADRs are **blocking** — agents cannot start work in an area governed by a `proposed` ADR. Work proceeds only after human acceptance. This ensures architectural decisions are deliberate, not accidental.

### Acceptance: Sealing the Lineage

When a human accepts completed work, they're not just approving code — they're **sealing the lineage**:

```
Story → implements → SRS requirement
SRS requirement → derives from → PRD requirement
PRD requirement → (optionally) → Bearing research
All → constrained by → ADRs
```

Acceptance confirms this chain is intact. The implementation traces back through requirements to architectural decisions.

## Bounded Contexts

Keel is organized into **bounded contexts** — distinct areas of the domain with clear boundaries:

| Context | Purpose | Core Entities |
|---------|---------|---------------|
| **governance** | ADR lifecycle, lineage tracking, constraint enforcement | ADR, ContextBinding, Lineage |
| **work-management** | Story/Voyage/Epic lifecycle, 2-queue system | Story, Voyage, Epic, Board |
| **research** | Bearing exploration and graduation | Bearing, Survey, Assessment |
| **visualization** | Read-side presentation | FlowMetrics, Bottleneck |
| **verification** | Artifact capture and tracing | VerificationResult |
| **learning** | Knowledge capture and propagation | Learning |

### Context Boundaries Enable Parallelization

Bounded contexts are not just code organization — they're the **unit of parallel work**.

When a voyage is decomposed into stories:
- Stories in **different contexts** with no dependencies → parallel safe
- Stories in the **same context** → batch sequential by default
- Stories with **sequential requirements** → forced sequential

The parallelization model **emerges from planning**, not from explicit orchestration. Good decomposition naturally reveals what can run in parallel.

## The Planning Pipeline

Work flows through a pipeline from exploration to completion:

```
[Bearing] --lay--> [Epic] --decompose--> [Voyage] --plan--> [Stories] --execute--> [Submit] --human verify--> [Done]
              \                                                                             ^
               \------------------------(optional research path)-----------------------------/
```

### Bearings (Optional)

Bearings are research artifacts for exploring new directions before committing to an epic. Not all epics require bearings — they're for when the path forward is uncertain.

### Zero-Allocation Planning

Like a well-designed program that allocates all memory at startup, **voyage decomposition allocates all work upfront**. When stories are created:

- Each story is a discrete unit of work
- Dependencies are explicit
- Bounded context membership is clear
- Parallelization opportunities are visible

Agents don't discover work at runtime — they execute pre-planned, pre-allocated work.

## The Engine Power Model

The engine operates like a circuit with two power sources:

```
╔═══════════════════════════════════════════════════════════════════════╗
║                         THE KEEL ENGINE                               ║
╠═══════════════════════════════════════════════════════════════════════╣
║                                                                       ║
║  [HUMAN] ════════════════════════════════════════╗  HIGH VOLTAGE      ║
║     │                                            ║  (Strategic)       ║
║     │    ┌──────────────────────────────────┐    ║                    ║
║     ├───►│ PHYSICS GATE (ADR)               │◄───╝                    ║
║     │    └──────────────┬───────────────────┘                         ║
║     │                   │                                             ║
║     │                   ▼                                             ║
║     │    ┌──────────────────────────────────┐                         ║
║     │    │ mission → voyage → story         │◄───╗                    ║
║     │    └──────────────┬───────────────────┘    ║                    ║
║     │                   │                        ║                    ║
║     │                   ▼                        ║  CONSTANT VOLTAGE  ║
║     │    ┌──────────────────────────────────┐    ║  (Implementation)  ║
║     ├───►│ STATE GATE (ACCEPT)              │    ║                    ║
║     │    └──────────────┬───────────────────┘    ║                    ║
║     │                   │                        ║                    ║
║     │                   ▼                        ║                    ║
║     │               [verified]                   ║                    ║
║     │                                            ║                    ║
║  [HUMAN]                                     [AGENT]                  ║
║                                                                       ║
╚═══════════════════════════════════════════════════════════════════════╝
```

**Agent voltage** is constant — when the physics allow, agents execute implementation. This pushes work through the pipeline.

**Human voltage** is high but strategic — humans author the physics (ADRs) and calibrate the state (Acceptance).

The simulator requires both:
- Without implementation voltage → no state is produced.
- Without strategic voltage → work accumulates at gates, and the simulation halts.

Gates are the key constraint. They're where human judgment is irreplaceable. Everything else flows with agent voltage alone.

## The Dashboard Model

The `keel flow` dashboard surfaces work lanes for each actor. Keel uses a **configurable lane topology** to route work based on the roles pulling from the board.

| Type | Role Family | Lane | Purpose |
|------|-------------|------|---------|
| **Management** | `manager` | `management` | Planning, triage, verification |
| **Delivery** | `operator` | `delivery` | Active implementation |

This topology is fully overridable in `keel.toml`. New lanes and roles can be added to customize the "high voltage" and "constant voltage" paths of the circuit.

Queue items fall into two categories:

| Type | Significance | Visibility |
|------|--------------|------------|
| **Flow generators** | Feed the pipeline | Always visible |
| **Flow gates** | Block the pipeline | Visible when blocking |

**Flow generators** are always shown, even at zero — they represent the ongoing rhythm of work:
- Research (bearings to explore)
- Accept (stories awaiting human verification)
- Start (voyages ready to begin)
- Decompose (voyages needing story breakdown)

**Flow gates** appear only when active — they represent decisions that block progress:
- Proposed ADRs (governance decisions needed)

This distinction matters: seeing "research: 0" prompts exploration. Seeing "proposed ADRs: 2" signals a blocker requiring human judgment before work can proceed in governed contexts.

The dashboard answers two questions:
1. **What can move?** — Flow generators show available work
2. **What's stuck?** — Flow gates show pending decisions

## Principles

### 1. Pull Over Push

Actors pull work when ready. No coordination overhead. No blocking on others.

### 2. Files As Truth

All state lives in markdown files with YAML frontmatter. Git is the audit log. No hidden databases.

### 3. Derived Flow State

System health is computed from entity states, not stored separately. The board's health is always consistent with its contents.

### 4. Agent-Friendly

Formats are parseable. Health is verifiable. Context surfaces automatically. Agents don't need to guess.

### 5. Minimal Transitions

State changes require explicit commands. No implicit inference. Every transition is intentional and traceable.

### 6. Human At The Right Level

Humans decide architecture (ADRs), approve scope (PRD/SRS), and accept completion. Everything between is agent territory.

### 7. Verification Closes The Loop

Automated verification confirms implementation matches requirements. When verification passes, acceptance is lightweight.

## Evolution

This constitution will evolve. When it does, capture the change in an ADR explaining why the philosophy shifted. The constitution is not immutable, but changes to it are significant and should be deliberate.

---

*"The goal is not to automate humans out of the loop, but to place human judgment where it's irreplaceable."*
