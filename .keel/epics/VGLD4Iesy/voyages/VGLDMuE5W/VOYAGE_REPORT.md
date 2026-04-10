# VOYAGE REPORT: Define Durable Session And Capability Interfaces

## Voyage Metadata
- **ID:** VGLDMuE5W
- **Epic:** VGLD4Iesy
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Define Session Wake Slice And Checkpoint Contract
- **ID:** VGLDQ7pnQ
- **Status:** done

#### Summary
Define the durable session contract that later runtime code can depend on. This story should make wake, replay, checkpoint, and selective slice semantics explicit so the session becomes a stable object outside any particular model context window.

#### Acceptance Criteria
- [x] The session contract names how a harness wakes a prior session, replays it, and resumes from checkpoints without relying on ad hoc prompt summaries [SRS-01/AC-01] <!-- verify: cargo test trace_recorders -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] Selective event-slice interrogation is explicit enough that later context and recovery stories can consume it without redefining replay semantics [SRS-01/AC-02] <!-- verify: cargo test trace_recorders -- --nocapture, SRS-01:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGLDQ7pnQ/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGLDQ7pnQ/EVIDENCE/ac-2.log)

### Promote Embedded Transit Recorder To Default Session Spine
- **ID:** VGLDQ8NnO
- **Status:** done

#### Summary
Promote the embedded transit-backed recorder path into the default session spine for the runtime. This story should bound how persistent session recording becomes the normal path without breaking local-first failure behavior.

#### Acceptance Criteria
- [x] The runtime defines a default recorder posture that uses the embedded transit-backed session spine instead of treating recording as optional metadata [SRS-02/AC-01] <!-- verify: cargo test trace_recorders -- --nocapture, SRS-02:start:end, proof: ac-1.log-->
- [x] Fallback and failure behavior remains bounded and operator-visible when the persistent session spine cannot be used [SRS-02/AC-02] <!-- verify: cargo test service_new_uses_persistent_trace_recorder_posture_by_default -- --nocapture, SRS-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGLDQ8NnO/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGLDQ8NnO/EVIDENCE/ac-2.log)

### Replace Provider Branching With Negotiated Capability Surface
- **ID:** VGLDQ92nP
- **Status:** done

#### Summary
Replace provider-name-specific runtime branching with a negotiated capability surface for shared planner, renderer, and tool-call behavior. This story should define the contract rather than finalizing every migration.

#### Acceptance Criteria
- [x] Shared planner/render/tool-call behavior resolves from capability descriptors wherever the behavior is conceptually common across providers [SRS-03/AC-01] <!-- verify: cargo test capability_surface_negotiates_shared_http_render_and_tool_call_behavior -- --nocapture, SRS-03:start:end, proof: ac-1.log-->
- [x] The capability surface is documented and testable enough that future providers can fit the harness without forking controller logic [SRS-03/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "capability surface|provider/model pair|planner tool-call|transport support|future provider|forking controller logic" README.md ARCHITECTURE.md CONFIGURATION.md src/infrastructure/providers.rs', SRS-03:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGLDQ92nP/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGLDQ92nP/EVIDENCE/ac-2.log)


