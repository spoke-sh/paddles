# Turn Loop And HTTP Inference Cleanup - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver the research epic that inventories in-process Sift model inference, HTTP model-client seams, lane concepts, and turn-loop phase boundaries. | board: VJZ0tpZQJ |
| MG-02 | Keep the initial cleanup bearing available as the cited decision frame for the research mission. | manual: bearing VJZ034dF2 ready with cited assessment |
| MG-03 | Present a migration recommendation and sealed-slice plan before implementation begins. | manual: human review of recommendation from bearing VJZ034dF2 and voyage VJZ14yp0U |

## Constraints

- Do not delete or rewrite runtime code before the recommendation is reviewed.
- Keep local-first behavior available through HTTP-hosted local model services rather than paddles-owned model loading unless a later ADR decides otherwise.
- Treat Sift-backed retrieval separately from Sift-backed model inference until the research proves they should move together.
- Preserve the model-owned reasoning contract: expose live capability surfaces and enforced constraints instead of replacing the turn loop with controller-authored pseudo-plans.
- Update owning docs in the same future implementation slices that change runtime behavior.

## Halting Rules

- DO NOT halt while the source inventory or migration recommendation is incomplete.
- YIELD to human before implementation after the cleanup recommendation is presented.
- HALT only after the human has reviewed the recommendation or directed the next implementation slice.
