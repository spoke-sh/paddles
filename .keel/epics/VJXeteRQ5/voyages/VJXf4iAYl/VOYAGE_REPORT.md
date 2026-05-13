# VOYAGE REPORT: Planner Schema Documentation

## Voyage Metadata
- **ID:** VJXf4iAYl
- **Epic:** VJXeteRQ5
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Document Shared Planner Action Schema Contract
- **ID:** VJXfKufl7
- **Status:** done

#### Summary
Update foundational documentation so the shared planner action schema renderer,
turn-specific capability manifest, and provider adapter responsibilities match
the implemented runtime.

#### Acceptance Criteria
- [x] README describes the shared schema renderer and turn-specific capability manifest split. [SRS-01/AC-01] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "shared planner action schema renderer" README.md && rg -n "turn-specific capability manifest" README.md', SRS-01:start:end, proof: ac-1.log-->
- [x] POLICY forbids adapter-local planner action schema lists. [SRS-02/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "Adapter-local planner action schema lists are forbidden" POLICY.md', SRS-02:start:end, proof: ac-2.log-->
- [x] ARCHITECTURE maps the shared renderer boundary and provider adapter responsibilities. [SRS-03/AC-03] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "src/application/planner_action_schema.rs" ARCHITECTURE.md && rg -n "Provider adapters still own provider-specific mechanics" ARCHITECTURE.md && rg -n "turn-specific capability manifest" ARCHITECTURE.md', SRS-03:start:end, proof: ac-3.log-->
- [x] Docs mention semantic actions and `external_capability` as part of the canonical schema when capability-disclosed. [SRS-04/AC-04] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "semantic_(definitions|references|symbols|hover|diagnostics)|semantic workspace actions" README.md ARCHITECTURE.md CONFIGURATION.md && rg -n "external_capability" README.md ARCHITECTURE.md CONFIGURATION.md', SRS-04:start:end, proof: ac-4.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJXfKufl7/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJXfKufl7/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJXfKufl7/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VJXfKufl7/EVIDENCE/ac-4.log)


