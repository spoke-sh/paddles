---
# system-managed
id: VHvKpVrcL
status: done
created_at: 2026-04-26T11:39:36
updated_at: 2026-04-26T11:51:26
# authored
title: Upgrade Web Ui Node Dependencies
type: chore
operator-signal:
scope: VHvKkR50r/VHvKoiGUZ
index: 1
started_at: 2026-04-26T11:41:05
completed_at: 2026-04-26T11:51:26
---

# Upgrade Web Ui Node Dependencies

## Summary

Refresh the npm workspace dependency graph so the reported web UI vulnerabilities are removed while the existing docs and web UI quality gates continue to pass.

## Acceptance Criteria

- [x] `npm audit` reports zero vulnerabilities for the workspace. [SRS-01/AC-01] <!-- verify: npm audit, SRS-01:start:end, proof: ac-1.log-->
- [x] Existing web and docs npm quality gates pass after the dependency refresh. [SRS-02/AC-01] <!-- verify: npm run lint && npm run test && npm run build && npm run e2e, SRS-02:start:end, proof: ac-2.log-->
