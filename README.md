# Paddles: Your Agentic SDLC Simulator Harness

[![Keel Board](https://img.shields.io/badge/Keel-Board-blue)](.keel/README.md)
[![Release Process](https://img.shields.io/badge/Release-Process-green)](RELEASE.md)
[![Philosophy](https://img.shields.io/badge/Philosophy-Article-orange)](PHILOSOPHY.md)

> Paddles is your specialized agentic harness, designed to operate within the high-fidelity simulation environment provided by the Keel Engine. It's the mech suit for your AI assistant, enabling turn-based coding tasks with unparalleled precision and verifiable outcomes.

`paddles` empowers AI assistants by leveraging the **Keel Engine**, a sophisticated SDLC simulator. Keel provides the underlying "physics" and "circuitry" for development, while `paddles` integrates agentic tools and workflows to execute within this framework. Together, they ensure that development adheres to defined architectural constraints, strategic intent, and verifiable specifications.

---

## ⚙️ The Keel Simulation Environment

Keel treats software development as a high-fidelity simulation where human judgment defines the physics and agents execute within that sandbox. This environment is governed by core principles:

*   **Core Belief:** Humans author the physics (architecture, scope), agents execute within the simulation (implementation, testing), and verification confirms the state.
*   **2-Queue Pull System:** A robust collaboration model where humans and agents pull work when ready, minimizing drift and maximizing throughput.
*   **Verified Spec Driven Development (VSDD):** Every transition is gated by verifiable evidence, ensuring implementation precisely matches requirements.
*   **Pacemaker Stability:** The system's heartbeat ensures continuous, stable operation, with autonomous "gardening" resolving drift.

---

## 🚀 Getting Started with Paddles & Keel

The development environment is managed by `nix`.

1.  **Enter the development shell:**
    ```bash
    nix develop
    ```
    This command installs all necessary dependencies, including the Rust toolchain, CUDA support, and other system libraries.

2.  **Ensure Board Integrity (Gardening First):**
    Before any other action, run the doctor to diagnose and fix any inconsistencies in the Keel board state:
    ```bash
    just keel doctor
    ```
    This is crucial for maintaining the integrity of the simulation.

3.  **Build the project:**
    ```bash
    just build
    ```

4.  **Run the tests:**
    ```bash
    just test
    ```

---

## 🧭 Navigating the Ramping Path

As you interact with the system, you'll naturally progress through these roles:

1.  **The Fixer (Learning by Healing):** Start by running `just keel doctor` to find and fix objective issues on the board. This is the best way to learn the structural invariants of the system.
2.  **The Operator (Learning by Building):** Once the board is healthy, implement stories using `just keel mission next --role operator`. You'll pull tasks, record evidence, and learn how requirements flow into verified code.
3.  **The Manager (Learning by Constraining):** At the highest level, define the physics of the sandbox by authoring ADRs, approving scope (Epics/Voyages), and managing the human-led decision gates.

---

## 🛠️ Core Tools & Components

`paddles` integrates these key tools within the Keel Engine:

*   **Agentic Capabilities:** `wonopcode` (AI coding assistant), `sift` (information retrieval), `candle` (local AI models) provide the intelligence for task execution.
*   **Keel Engine:** Manages the SDLC simulation, state transitions, verification, and collaboration queues.
*   **Foundational Documents:** The philosophy and constraints are defined in the documents listed below.

---

## 📚 Foundational Documents

The philosophy and constraints of this repository are defined in the following documents. New contributors should read them to understand the rules of the game.

-   [PHILOSOPHY.md](PHILOSOPHY.md): The core principles of the Agentic SDLC Simulator.
-   [CONSTITUTION.md](CONSTITUTION.md): The philosophy of collaboration and decision hierarchy.
-   [AGENTS.md](AGENTS.md): Tactical guidance for AI contributors.
-   [INSTRUCTIONS.md](INSTRUCTIONS.md): Step-by-step procedural loops and checklists.
-   [ARCHITECTURE.md](ARCHITECTURE.md): Implementation architecture and flow model.
-   [CONFIGURATION.md](CONFIGURATION.md): Role-based and config-driven topology.
-   [EVALUATIONS.md](EVALUATIONS.md): The evaluation strategy and artifact model.
-   [RELEASE.md](RELEASE.md): The release process and artifacts.
-   [.keel/adrs/](.keel/adrs/) — Binding architecture decisions.
