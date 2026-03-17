# Paddles Architecture: The Mech Suit

This document describes the internal architecture of `paddles`, the agentic harness and "mech suit" for AI assistants.

## System Overview

`paddles` acts as a high-fidelity interface between human intent and autonomous execution. It coordinates the boot sequence calibration, manages the agentic loop through `wonopcode-core`, and provides local inference capacity via `candle`.

## The Boot Sequence (Calibration)

Before execution begins, the system must be calibrated to its human environment.

1.  **Inheritance**: Initial credit balance loaded via `--credits`.
2.  **Environmental Weights**: Floating-point weights and biases applied to tune agent behavior.
3.  **Constitutional Validation**: Weights are checked against defined min/max bounds.
4.  **Religious Dogma**: Immutable invariants (e.g., "Simulation over Reality") are enforced. Any violation results in an **Unclean Boot**.

## Component Stack

### 1. CLI Entry (`src/main.rs`)
- Parses arguments via `clap`.
- Orchestrates the `BootContext` and `PromptLoop`.
- Handles both single-turn `--prompt` and multi-turn interactive mode.

### 2. Boot Engine (`BootContext`)
- Encapsulates the `Constitution` and `Dogma` logic.
- Maintains the state of credits, weights, and biases.

### 3. Execution Engine (`wonopcode-core`)
- **Instance**: Manages project-level state and configurations.
- **Session**: Provides temporal context for interactions.
- **PromptLoop**: Orchestrates the multi-turn agentic interaction.

### 4. Local Brain (`CandleProvider`)
- Implements the `LanguageModel` trait from `wonopcode-provider`.
- Targeted for zero-network, local execution using `candle-transformers`.

## Verified Spec Driven Development (VSDD)

`paddles` manages its own development using the Keel engine. All tactical moves are tracked in the `.keel/` directory:
- **Missions**: Long-term strategic objectives.
- **Epics**: Large value-shifted features.
- **Voyages**: Technical implementation paths (SRS/SDD).
- **Stories**: Atomic work units verified by proofs (`[SRS-XX/AC-YY]`).

## Data Flow

1. **Invoke**: User runs `just paddles` or `paddles --prompt`.
2. **Calibrate**: `BootContext` validates the environment.
3. **Initialize**: `Instance` and `Session` are established.
4. **Loop**: `PromptLoop` coordinates between the `CandleProvider` and the user.
5. **Output**: Text deltas are rendered to the terminal.
