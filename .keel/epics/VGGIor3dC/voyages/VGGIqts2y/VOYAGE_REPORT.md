# VOYAGE REPORT: Simplify Forensic Inspector Around Machine Narrative

## Voyage Metadata
- **ID:** VGGIqts2y
- **Epic:** VGGIor3dC
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Collapse Forensic Selection To Moments And Internals
- **ID:** VGGIuWphu
- **Status:** done

#### Summary
Replace the current forensic conversation/turn/record selection maze with the same turn + moment + internals model used by the transit machine.

#### Acceptance Criteria
- [x] The forensic route adopts the shared turn + moment + internals selection model instead of route-specific selection semantics. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] The default forensic selection path collapses operator choices instead of preserving the current conversation/turn/record mode maze. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-2.log-->
- [x] Default forensic navigation no longer requires separate conversation/turn/record mode switching to understand a turn. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGGIuWphu/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGGIuWphu/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VGGIuWphu/EVIDENCE/ac-3.log)

### Retire Legacy Forensic Nav List And Pane
- **ID:** VGGIuXNif
- **Status:** done

#### Summary
Retire the legacy forensic nav/list/detail composition once the narrative machine surface and detail drawer provide the primary operator path.

#### Acceptance Criteria
- [x] Legacy forensic nav/list/detail chrome is removed or demoted so the machine stage and drawer become the default comprehension path. [SRS-NFR-01/AC-01] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-1.log-->
- [x] Route tests and authored docs are updated to describe the new default forensic workflow and its explicit internals escape hatch. [SRS-NFR-02/AC-02] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGGIuXNif/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGGIuXNif/EVIDENCE/ac-2.log)

### Build Forensic Machine Detail Drawer
- **ID:** VGGIuXjjA
- **Status:** done

#### Summary
Build a focused forensic detail drawer that explains the selected machine moment, its steering forces, and any before/after artifact context.

#### Acceptance Criteria
- [x] The forensic route renders a machine-moment detail surface that explains why the selected moment mattered before exposing raw payloads. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] Internals mode still exposes raw payloads, record ids, and evidence links without dominating the default detail presentation. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [x] The detail surface exposes an explicit internals path for raw payloads, record ids, and comparison context without restoring the old always-on pane composition. [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGGIuXjjA/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGGIuXjjA/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VGGIuXjjA/EVIDENCE/ac-3.log)


