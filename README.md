# Paddles
[![Keel Board](https://img.shields.io/badge/Keel-Board-blue)](.keel/README.md)
[![Release Process](https://img.shields.io/badge/Release-Process-green)](RELEASE.md)

> The mech suit for the famous assistant, Paddles mate!

`paddles` is a Rust-native agentic harness designed to be a reliable and powerful "mech-suit" for AI assistants. It provides a robust framework for creating and managing agentic workflows, with a focus on turn-based coding tasks. This project is managed by the `keel` engine, which treats software development as a high-fidelity simulation.

---

## 🎮 The Ramping Path

Like other `keel`-based projects, `paddles` is designed to meet you where you are. As you interact with the engine, you'll naturally level up through three distinct roles:

1.  **The Fixer (Learning by Healing)**: Start by running `keel doctor` to find and fix objective issues on the board. This is the best way to learn the structural invariants of the system.
2.  **The Operator (Learning by Building)**: Once the board is healthy, move into implementation with `keel mission next --status`. You'll pull stories, record evidence, and learn how requirements flow from planning into verified code.
3.  **The Architect (Learning by Constraining)**: At the highest level, you'll define the physics of the sandbox by authoring Architecture Decision Records (ADRs) and tactical plans.

---

## ⚙️ Core Components

`paddles` integrates a suite of powerful tools to achieve its goals:

- **`wonopcode`**: A high-performance, AI-powered coding assistant that provides the core agentic capabilities.
- **`sift`**: A hybrid information retrieval system used to "retrieve and emit matter with gravity and fusion ejection."
- **`candle`**: A minimalist ML framework for running local AI models, enabling powerful on-device capabilities.
- **`keel`**: A project management and build system that ensures the project stays on track and maintains high quality.

## 📚 Foundational Documents

The philosophy and constraints of this repository are defined in the following documents. New contributors should read them to understand the rules of the game.

-   [CONSTITUTION.md](CONSTITUTION.md): The philosophy of collaboration.
-   [AGENTS.md](AGENTS.md): The tactical loop for AI contributors.
-   [INSTRUCTIONS.md](INSTRUCTIONS.md): Step-by-step procedural loops and checklists.
-   [ARCHITECTURE.md](ARCHITECTURE.md): Implementation architecture and flow model.
-   [CONFIGURATION.md](CONFIGURATION.md): Role-based and config-driven topology.
-   [EVALUATIONS.md](EVALUATIONS.md): The evaluation strategy and artifact model.
-   [RELEASE.md](RELEASE.md): The release process and artifacts.

---

## 🚀 Getting Started

The development environment is managed by `nix`.

1.  **Enter the development shell:**
    ```bash
    nix develop
    ```
    This command will download and install all the necessary dependencies, including the Rust toolchain, CUDA support, and other system libraries.

2.  **Build the project:**
    ```bash
    just build
    ```

3.  **Run the tests:**
    ```bash
    just test
    ```
