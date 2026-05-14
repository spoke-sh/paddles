---
# system-managed
id: VJXwmpZXz
status: done
created_at: 2026-05-13T16:37:36
updated_at: 2026-05-13T17:27:32
# authored
title: Update Foundational Docs For Recursive Agent Loop
type: feat
operator-signal:
scope: VJXwbmekZ/VJXwlG70U
index: 2
started_at: 2026-05-13T17:17:50
completed_at: 2026-05-13T17:27:32
---

# Update Foundational Docs For Recursive Agent Loop

## Summary

Update foundational documentation so Paddles is described as one recursive
agent loop where model reasoning is planning through bounded actions.

## Acceptance Criteria

- [x] README, POLICY, ARCHITECTURE, and CONFIGURATION state that model reasoning is the planning inside the recursive agent loop. [SRS-03/AC-01] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "model reasoning is the planning|recursive agent loop" README.md POLICY.md ARCHITECTURE.md CONFIGURATION.md', SRS-03:start:end, proof: ac-1.log-->
- [x] Foundational docs no longer describe direct answers as a pre-loop route outside the recursive agent loop. [SRS-NFR-01/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && ! rg -n "pre-loop routing|outside the recursive agent loop" README.md POLICY.md ARCHITECTURE.md CONFIGURATION.md', SRS-NFR-01:start:end, proof: ac-2.log-->
- [x] Docs describe terminal `answer`/`stop`, workspace actions, semantic actions, and `external_capability` as one recursive action vocabulary gated by the capability manifest. [SRS-04/AC-03] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "terminal.*answer.*stop|semantic actions|external_capability|capability manifest" README.md POLICY.md ARCHITECTURE.md CONFIGURATION.md', SRS-04:start:end, proof: ac-3.log-->
