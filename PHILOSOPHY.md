# PHILOSOPHY

This document captures the foundational principles of how Keel operates as an **Agentic SDLC Simulator**. It defines the human-agent collaboration model, the resolution hierarchy, and the formal boundaries that keep autonomous delivery aligned with strategic intent.

## Core Belief: Humans Author Physics, Agents Execute Simulation, Verification Confirms State

The fundamental principle guiding Keel is that human judgment is irreplaceable at the highest levels of strategy and constraint definition. Humans author the "physics" of the software development simulation by defining architectural decisions (ADRs) and approving scope (Epics, Voyages). Agents then execute tactical tasks (Stories) within these defined constraints, operating as high-fidelity simulators. Verification confirms that the simulated execution accurately reflects the intended state. The goal is not to remove humans from software development, but to place their judgment and strategic direction where it is most impactful.

## Collaboration Model: The 2-Queue Pull System

Keel fosters a highly autonomous yet human-guided development process through a **2-Queue Pull System**.

*   **Human Responsibilities:** Architectural decisions, scope approval, acceptance/rejection, research direction, defining the "physics" of the simulation.
*   **Agent Responsibilities:** Tactical execution (implementation, testing, verification), documentation, adherence to defined constraints.

Actors (humans and agents) pull work from their respective queues when they are ready, minimizing coordination overhead and maximizing throughput. This model ensures that human oversight is focused on strategic alignment, while autonomous agents handle the bulk of the execution.

## Decision Hierarchy: From Constitution to Execution

Decisions and constraints flow through a strict hierarchy, ensuring that tactical execution remains aligned with strategic intent:

1.  **ADRs (Architecture Decision Records):** Define the binding architectural "physics" and constraints. These are paramount and govern all subsequent decisions.
2.  **CONSTITUTION:** Captures the philosophy of collaboration, the simulation model, and the high-level principles of interaction.
3.  **FORMAL RULES:** Define the executable engine invariants and operational constraints.
4.  **ARCHITECTURE:** Outlines the physical source layout, dependency boundaries, and runtime flows.
5.  **PLANNING (PRD/SRS/SDD):** Defines the scoped mission work, requirements, and tactical plans that agents execute.

This hierarchy ensures that all work, from strategic planning to tactical story execution, is tightly bound to the project's architectural vision and philosophical underpinnings.

## The Simulator's Goal: Minimize Drift, Maximize Fidelity

The overarching goal of the Keel simulator is to minimize "drift" – the divergence between intended state and actual state. This is achieved by:

*   **High-Fidelity Simulation:** Agents execute work within precisely defined constraints, mimicking real-world development processes.
*   **Verifiable State:** All transitions and work products are backed by verifiable evidence, ensuring the state of the system is always consistent with its contents.
*   **Strategic Human Oversight:** Human judgment is applied at critical decision points (architecture, scope, acceptance) to guide the simulation's direction.

By adhering to these principles, Keel aims to provide a development environment where autonomous agents can operate with maximum efficiency and reliability, guided by clear human intent and verifiable outcomes.
