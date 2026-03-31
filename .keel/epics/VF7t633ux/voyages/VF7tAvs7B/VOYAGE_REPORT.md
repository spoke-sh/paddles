# VOYAGE REPORT: Sift-Native Runtime Cutover

## Voyage Metadata
- **ID:** VF7tAvs7B
- **Epic:** VF7t633ux
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Build Sift Session Controller
- **ID:** VF7tCKEgw
- **Status:** done

#### Summary
Replace the legacy-engine-owned prompt loop with a Paddles-managed Sift session
controller that owns conversational state and retained context.

#### Acceptance Criteria
- [x] `MechSuitService` executes prompts through a Sift session controller rather than `legacy_core::PromptLoop` and `Instance`. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] Prompt turns retain prior agent turns and bounded workspace evidence through Sift context state. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VF7tCKEgw/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VF7tCKEgw/EVIDENCE/ac-2.log)
- [ac1-runtime-cutover.md](../../../../stories/VF7tCKEgw/EVIDENCE/ac1-runtime-cutover.md)
- [ac2-retained-context.md](../../../../stories/VF7tCKEgw/EVIDENCE/ac2-retained-context.md)

### Add Local Tool Surface
- **ID:** VF7tCKUgx
- **Status:** done

#### Summary
Add the initial local coding tool surface so the Sift-native runtime can inspect,
search, and mutate a workspace without depending on legacy-engine tool plumbing.

#### Acceptance Criteria
- [x] The runtime exposes immediate local tools for search, file operations, shell commands, and edit/diff operations. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] Tool results are recorded as searchable local context for later turns. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VF7tCKUgx/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VF7tCKUgx/EVIDENCE/ac-2.log)
- [ac1-local-tool-surface.md](../../../../stories/VF7tCKUgx/EVIDENCE/ac1-local-tool-surface.md)
- [ac2-tool-output-context.md](../../../../stories/VF7tCKUgx/EVIDENCE/ac2-tool-output-context.md)

### Cut Over Runtime And Docs
- **ID:** VF7tCKsgv
- **Status:** done

#### Summary
Cut the CLI and repository boundary over to the new Sift-native runtime,
removing legacy-engine from core execution and updating the authored docs.

#### Acceptance Criteria
- [x] Single-prompt and interactive CLI flows remain operational after the cutover. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] legacy-core/provider/tools are removed from core runtime modules and Cargo runtime dependencies. [SRS-NFR-01/AC-01] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-2.log-->
- [x] Verbose output exposes context assembly and tool execution clearly enough to debug the controller. [SRS-NFR-02/AC-01] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VF7tCKsgv/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VF7tCKsgv/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VF7tCKsgv/EVIDENCE/ac-3.log)
- [ac1-cli-cutover.md](../../../../stories/VF7tCKsgv/EVIDENCE/ac1-cli-cutover.md)
- [ac2-runtime-boundary.md](../../../../stories/VF7tCKsgv/EVIDENCE/ac2-runtime-boundary.md)
- [ac3-verbose-debugging.md](../../../../stories/VF7tCKsgv/EVIDENCE/ac3-verbose-debugging.md)


