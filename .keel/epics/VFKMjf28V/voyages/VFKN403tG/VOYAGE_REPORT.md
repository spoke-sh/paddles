# VOYAGE REPORT: Provider Routing And CLI Flags

## Voyage Metadata
- **ID:** VFKN403tG
- **Epic:** VFKMjf28V
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Provider CLI Flag And Factory Routing
- **ID:** VFKN7LdAb
- **Status:** done

#### Summary
Add --provider CLI flag and route factory closures to the correct adapter constructor based on provider selection, with API key resolution from environment variables.

#### Acceptance Criteria
- [x] --provider flag accepts local, openai, anthropic, google, moonshot [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [x] Factory closures dispatch to correct adapter based on provider [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [x] API key resolved from provider-specific env var [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
- [x] --provider-url overrides default API base URL [SRS-04/AC-04] <!-- verify: manual, SRS-04:start:end -->
- [x] Local provider is default when --provider omitted [SRS-05/AC-05] <!-- verify: manual, SRS-05:start:end -->


