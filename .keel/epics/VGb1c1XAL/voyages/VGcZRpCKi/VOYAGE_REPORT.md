# VOYAGE REPORT: Establish A Typed External Capability Fabric Substrate

## Voyage Metadata
- **ID:** VGcZRpCKi
- **Epic:** VGb1c1XAL
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Define External Capability Contracts
- **ID:** VGcZTFufQ
- **Status:** done

#### Summary
Define the typed capability-descriptor, invocation, result, and governance
contracts for web, MCP, and connector-backed actions so the recursive harness
can reason about external capability use through one vocabulary.

#### Acceptance Criteria
- [x] The runtime defines typed contracts for external capability descriptors, invocation intents and results, availability metadata, auth posture, and evidence expectations across web, MCP, and connector-backed actions. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] The external capability contract stays generic enough to absorb new fabrics without reworking the recursive planner contract. [SRS-NFR-04/AC-01] <!-- verify: manual, SRS-NFR-04:start:end, proof: ac-2.log-->
- [x] The contract vocabulary composes with existing execution-governance and evidence-first boundaries instead of introducing surface-specific client paths. [SRS-NFR-03/AC-01] <!-- verify: manual, SRS-NFR-03:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGcZTFufQ/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGcZTFufQ/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VGcZTFufQ/EVIDENCE/ac-3.log)

### Route External Capability Discovery Through The Recursive Harness
- **ID:** VGcZTGZgd
- **Status:** done

#### Summary
Teach the recursive planner and runtime loop to discover, select, and invoke
external capabilities through the same action flow used for local workspace and
shell work, including the first pass that normalizes external results into the
evidence-first runtime.

#### Acceptance Criteria
- [x] The planner and runtime can discover external capabilities and invoke them through the same recursive action loop used for local work. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] External results normalize into evidence items, source records, and runtime artifacts with lineage, summaries, and availability state instead of remaining opaque tool output. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] External capability invocation composes with auth, approval, and sandbox governance instead of bypassing the local execution policy model. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end, proof: ac-3.log-->
- [x] The recursive harness remains useful when all external capability fabrics are absent or disabled. [SRS-NFR-01/AC-01] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-4.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGcZTGZgd/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGcZTGZgd/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VGcZTGZgd/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VGcZTGZgd/EVIDENCE/ac-4.log)

### Project External Capability Evidence And Degradation Across Surfaces
- **ID:** VGcZTH2gV
- **Status:** done

#### Summary
Project external capability state into trace and operator surfaces so active
fabrics, degraded outcomes, and provenance remain legible across TUI, web, API,
and docs once the recursive harness can already negotiate and normalize those
results.

#### Acceptance Criteria
- [x] Tool absence, auth failure, or stale capability metadata degrades honestly with explicit runtime state and no false success. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] TUI, web, and API projections plus operator docs expose active fabrics, external result provenance, and degraded states using one shared vocabulary. [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end, proof: ac-2.log-->
- [x] External capability metadata and results remain observable through trace, transcript, and API surfaces. [SRS-NFR-02/AC-01] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGcZTH2gV/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGcZTH2gV/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VGcZTH2gV/EVIDENCE/ac-3.log)


