# Registry Stabilization - Charter

Archetype: Maintenance

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Switch default model to non-gated Qwen-1.5B | board: VE5x7tOAX |
| MG-02 | Implement Hugging Face token support via env/CLI | board: VE5x7tOAX |

## Constraints

- Calibration must remain clean.
- Gated models must only work when a token is provided.

## Halting Rules

- HALT when `just paddles` succeeds without manual token input for the default model.
