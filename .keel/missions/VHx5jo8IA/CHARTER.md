# Add GPT-5.5 OpenAI Model Support - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Operators can select the current OpenAI GPT-5.5 family and supported OpenAI text/reasoning pro model IDs through Paddles' provider catalog with correct Responses routing. | board: VHx5jpzIB |

## Constraints

- Preserve local-first execution: provider catalog changes must not add new runtime network dependencies beyond the existing OpenAI HTTP adapter.
- Keep model capability behavior centralized in the provider capability surface.
- Exclude non-chat/non-reasoning media models from this coding-harness catalog slice unless a dedicated adapter supports them.

## Halting Rules

- Continue until epic VHx5jpzIB has all stories accepted, the voyage is done, and the mission is achieved or verified.
- Stop and escalate only if official OpenAI model documentation contradicts the requested catalog scope or if the existing OpenAI HTTP adapter cannot safely route a requested model family.
- Do not change default lane selections as part of this mission.
