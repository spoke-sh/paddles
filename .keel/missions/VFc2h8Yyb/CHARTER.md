# Add Inception Provider Support - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Plan and stage first-class Inception Labs provider support so paddles can land Mercury-2 chat compatibility first and then pursue provider-native diffusion and edit capabilities in explicit follow-on slices. | board: VFc2hwU7e |

## Constraints

- Reuse the existing remote-provider and OpenAI-compatible HTTP seams for core Mercury-2 support rather than inventing a new runtime path.
- Do not require upstream `sift` changes for the remote-provider onboarding slice.
- Keep streaming/diffusion visualization and edit-native endpoints as explicit follow-on slices so they do not block the core Mercury-2 integration.

## Halting Rules

- DO NOT halt while epic `VFc2hwU7e` or its child voyage/stories still contain unplanned or unfinished work.
- HALT when epic `VFc2hwU7e` is verified and the Inception provider plan clearly separates Mercury-2 core support from optional streaming/diffusion and edit-native follow-on slices.
- YIELD if supporting Inception would require architectural changes outside paddles' existing remote-provider boundary or an upstream `sift` dependency for the core Mercury-2 slice.
