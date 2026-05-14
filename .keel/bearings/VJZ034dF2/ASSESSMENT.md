---
id: VJZ034dF2
---

# Map Turn Loop And HTTP Inference Cleanup — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | The cleanup would remove a major architectural split between in-process model hosting and HTTP model clients, and would align public runtime concepts around the turn loop [SRC-01] [SRC-05] [SRC-06]. |
| Confidence | 4 | The current code has identifiable seams for Sift model preparation, Sift planner/synthesizer adapters, HTTP model adapters, and prepared runtime lane state [SRC-02] [SRC-03] [SRC-04] [SRC-05]. |
| Effort | 4 | The work spans source, tests, CLI/config compatibility, provider capability surfaces, and foundational docs rather than a small rename [SRC-04] [SRC-07]. |
| Risk | 4 | Earlier missions intentionally shipped local Sift inference and lane-specific routing, so deletion without ADR/test-backed migration could break expected local-first behavior [SRC-02] [SRC-03] [SRC-07]. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Findings

- Proceeding is justified, but the first active slice should remain research and
  architecture mapping rather than deleting Sift code immediately [SRC-01]
  [SRC-02] [SRC-03].
- HTTP-only inference has an existing migration seam in `src/main.rs`, where
  Sift planner/synthesizer factories can be replaced by HTTP-backed factories
  while preserving provider wire-format negotiation [SRC-05].
- The turn loop already owns the important runtime behavior, so lane collapse
  should reduce public configuration and naming while keeping tested phase
  boundaries around action selection, retrieval, execution, and synthesis
  [SRC-04] [SRC-06].
- Sift retrieval and Sift model inference must be separated explicitly. The
  cleanup can remove Sift as a model provider without necessarily removing the
  `sift-direct` retrieval/index backend in the same slice [SRC-04] [SRC-05].
- Documentation is part of the cleanup surface because CONFIGURATION currently
  teaches local Sift models and lane-specific operation [SRC-07].

## Opportunity Cost

This will delay feature work while the runtime vocabulary and inference boundary
are stabilized. That cost is acceptable because the current code and docs encode
legacy assumptions that will keep multiplying migration work if left in place
[SRC-04] [SRC-07].

## Dependencies

- A source inventory that distinguishes Sift-as-model-provider from Sift-as-
  retrieval-backend before deletion work begins [SRC-02] [SRC-03] [SRC-04].
- A migration map that identifies the first testable implementation slice,
  preferably the HTTP-only model-provider boundary before broader lane collapse
  [SRC-05].
- A decision on whether to record an ADR for deleting in-process local model
  hosting from paddles [SRC-01] [SRC-02] [SRC-03].
- Foundational documentation updates in the same sealed slices that change
  runtime behavior or operator configuration [SRC-07].

## Alternatives Considered

- Keep Sift local model inference and only rename lanes: rejected because it
  leaves model loading, hardware residency, and inference lifecycle inside
  paddles, which is the core concern raised by the cleanup [SRC-01] [SRC-02]
  [SRC-03].
- Delete every Sift reference immediately: rejected because `sift-direct`
  retrieval may still be a legitimate local index backend even if Sift is
  removed as a model provider [SRC-04] [SRC-05].
- Collapse planner/synthesizer/gatherer by merging all code into one runtime
  object: rejected because the turn loop already centralizes orchestration, and
  the better cleanup is public-concept simplification with internal phase
  boundaries preserved [SRC-06].

## Recommendation

[x] Proceed: keep mission active and complete source inventory plus migration map before implementation [SRC-01] [SRC-05] [SRC-06]
[ ] Park → revisit later [SRC-07]
[ ] Decline → document learnings [SRC-02] [SRC-03]
