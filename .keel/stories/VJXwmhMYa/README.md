---
# system-managed
id: VJXwmhMYa
status: done
created_at: 2026-05-13T16:37:36
updated_at: 2026-05-13T16:53:31
# authored
title: Render One Agent Action Schema For First And Later Steps
type: feat
operator-signal:
scope: VJXwbmekZ/VJXwlCA0P
index: 2
started_at: 2026-05-13T16:49:51
completed_at: 2026-05-13T16:53:31
---

# Render One Agent Action Schema For First And Later Steps

## Summary

Migrate the shared action schema renderer so first and later decision prompts
come from one canonical agent action vocabulary with variant-specific
availability.

## Acceptance Criteria

- [x] The shared schema renderer derives first and later variants from one action-entry source, with `answer` available as a terminal action only where the variant permits it. [SRS-03/AC-01] <!-- verify: cargo test agent_action_schema_variants_share_one_entry_source --lib, SRS-03:start:end, proof: ac-1.log-->
- [x] Schema parity tests compare rendered action names to the unified agent action contract and `WorkspaceAction`, including semantic actions and `external_capability`. [SRS-04/AC-02] <!-- verify: cargo test agent_action_schema_matches_domain_contract --lib, SRS-04:start:end, proof: ac-2.log-->
- [x] Sift and HTTP mocked prompt tests still prove both lanes receive the same marker-bounded schema block for first and later loop decisions. [SRS-04/AC-03] <!-- verify: cargo test agent_action_schema_prompt_parity --lib, SRS-04:start:end, proof: ac-3.log-->
