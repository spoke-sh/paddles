# VOYAGE REPORT: Auth and Default Stabilization

## Voyage Metadata
- **ID:** VE5xDYVO9
- **Epic:** VE5x7tOAX
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Stabilize Registry Access
- **ID:** VE5xHxg36
- **Status:** done

#### Summary
Switch the default model to a non-gated one and add support for Hugging Face authentication tokens.

#### Acceptance Criteria
- [x] Default model is set to `qwen-1.5b`. [SRS-23/AC-01] <!-- verify: manual, SRS-23:start:end -->
- [x] CLI accepts `--hf-token` argument. [SRS-24/AC-01] <!-- verify: manual, SRS-24:start:end -->
- [x] `HFHubAdapter` uses the provided token for requests. [SRS-25/AC-01] <!-- verify: manual, SRS-25:start:end -->
- [x] Token is never printed to logs or stdout. [SRS-NFR-10/AC-01] <!-- verify: manual, SRS-NFR-10:start:end -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VE5xHxg36/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VE5xHxg36/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VE5xHxg36/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VE5xHxg36/EVIDENCE/ac-4.log)


