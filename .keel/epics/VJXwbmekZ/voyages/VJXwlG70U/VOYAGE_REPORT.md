# VOYAGE REPORT: Agent Vocabulary Cleanup

## Voyage Metadata
- **ID:** VJXwlG70U
- **Epic:** VJXwbmekZ
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 2/2 stories complete

## Implementation Narrative
### Migrate Adapter Prompts To Agent Loop Vocabulary
- **ID:** VJXwmnTaa
- **Status:** done

#### Summary
Update Sift/local and HTTP/remote planner-facing prompts so they describe one
recursive agent loop and one bounded action-selection task, including the first
action.

#### Acceptance Criteria
- [x] Sift and HTTP prompts describe the model as selecting bounded recursive agent actions inside the loop, not as choosing whether to enter a separate planner phase. [SRS-01/AC-01] <!-- verify: cargo test agent_loop_prompt_vocabulary --lib, SRS-01:start:end, proof: ac-1.log-->
- [x] Prompt tests fail if either lane reintroduces adapter-local first-action or recursive-action schema drift. [SRS-02/AC-02] <!-- verify: cargo test agent_loop_prompt_schema_parity --lib, SRS-02:start:end, proof: ac-2.log-->
- [x] Prompt tests keep terminal `answer`/`stop`, workspace actions, semantic actions, and `external_capability` in the unified action vocabulary gated by the capability manifest. [SRS-02/AC-03] <!-- verify: cargo test agent_loop_prompt_capability_manifest_split --lib, SRS-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJXwmnTaa/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJXwmnTaa/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJXwmnTaa/EVIDENCE/ac-3.log)

### Update Foundational Docs For Recursive Agent Loop
- **ID:** VJXwmpZXz
- **Status:** done

#### Summary
Update foundational documentation so Paddles is described as one recursive
agent loop where model reasoning is planning through bounded actions.

#### Acceptance Criteria
- [x] README, POLICY, ARCHITECTURE, and CONFIGURATION state that model reasoning is the planning inside the recursive agent loop. [SRS-03/AC-01] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "model reasoning is the planning|recursive agent loop" README.md POLICY.md ARCHITECTURE.md CONFIGURATION.md', SRS-03:start:end, proof: ac-1.log-->
- [x] Foundational docs no longer describe direct answers as a pre-loop route outside the recursive agent loop. [SRS-NFR-01/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && ! rg -n "pre-loop routing|outside the recursive agent loop" README.md POLICY.md ARCHITECTURE.md CONFIGURATION.md', SRS-NFR-01:start:end, proof: ac-2.log-->
- [x] Docs describe terminal `answer`/`stop`, workspace actions, semantic actions, and `external_capability` as one recursive action vocabulary gated by the capability manifest. [SRS-04/AC-03] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "terminal.*answer.*stop|semantic actions|external_capability|capability manifest" README.md POLICY.md ARCHITECTURE.md CONFIGURATION.md', SRS-04:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJXwmpZXz/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJXwmpZXz/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJXwmpZXz/EVIDENCE/ac-3.log)


