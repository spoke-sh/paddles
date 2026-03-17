# Docking with Sift - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Replace manual HF/Candle logic with `sift` adapters | board: VE5zxrA1w |
| MG-02 | Execute multi-turn prompts using `sift` backed models | board: VE5zxrA1w |

## Constraints

- Use `sift::internal` components for model acquisition and inference.
- Maintain DDD/Hexagonal integrity.

## Halting Rules

- HALT when `paddles` successfully generates text using `sift`'s internal Qwen/Gemma logic.
