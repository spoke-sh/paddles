# Integrate Provider Reasoning Across The Recursive Harness - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver an active epic slice that integrates provider-native reasoning across every supported model provider by adding adapter-owned deliberation substrates, normalized harness signals, and explicit boundaries between raw provider reasoning and paddles-authored rationale. | board: VHXJWQaFC |

## Constraints

- Preserve paddles `rationale` as the canonical, provider-agnostic justification for harness decisions. Raw provider reasoning must not replace it in transcripts, projections, or durable turn records.
- Keep provider-native reasoning state adapter-owned or application-normalized. Provider-specific artifacts such as reasoning items, thinking blocks, thought signatures, summaries, or local think toggles must not leak into the domain core.
- Every supported provider in `ModelProvider::ALL` (`sift`, `openai`, `inception`, `anthropic`, `google`, `moonshot`, `ollama`) must negotiate an explicit deliberation capability and an explicit thinking-mode surface, including unsupported or no-op behavior where native continuation or configurable thinking is unavailable.
- Preserve local-first runtime constraints. Do not introduce hosted orchestration, remote state services, or non-optional cloud dependencies to make deliberation continuity work.
- Keep raw provider reasoning out of canonical render/replay truth. If recorded at all, it must live in a bounded debug or forensic path with clear retention and redaction limits.
- Provider-specific continuation semantics must terminate at infrastructure adapters or explicit application normalization seams, not in UI presentation logic or read-model reconstruction.
- Update the owning docs, configuration guidance, and board artifacts in the same slices that change provider reasoning behavior or rationale boundaries.

## Halting Rules

- DO NOT halt while epic `VHXJWQaFC` has any draft, planned, active, or verification-pending voyage/story work.
- HALT when epic `VHXJWQaFC` is complete and the runtime has board-linked evidence that every supported provider negotiates explicit reasoning behavior and thinking modes, normalized deliberation signals can influence the recursive harness, and paddles rationale remains distinct from provider-native reasoning.
- YIELD to the human if the remaining decision requires product direction on exposing provider reasoning to operators, retaining raw provider thought artifacts beyond debug scope, or changing the user-facing semantics of rationale and trace surfaces.
