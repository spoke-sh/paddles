# Local Neural Lattice - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Implement real local inference in `CandleProvider` | board: VE5jWMShq |
| MG-02 | Execute a prompt with zero network dependency | board: VE5jWMShq |

## Constraints

- Must use `candle-core` and `candle-transformers`.
- Must support quantized models for performance.

## Halting Rules

- HALT when `CandleProvider` generates text from a local model.
- YIELD if hardware acceleration (CUDA/Metal) setup blocks implementation.
