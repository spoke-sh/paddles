---
# system-managed
id: VFc2mykgg
status: done
created_at: 2026-04-01T23:31:01
updated_at: 2026-04-02T06:07:04
# authored
title: Wire Mercury-2 Through The OpenAI-Compatible HTTP Adapter
type: feat
operator-signal:
scope: VFc2hwU7e/VFc2jHVLG
index: 2
started_at: 2026-04-02T06:04:04
completed_at: 2026-04-02T06:07:04
---

# Wire Mercury-2 Through The OpenAI-Compatible HTTP Adapter

## Summary

Reuse the existing OpenAI-compatible HTTP adapter to execute `mercury-2`
end-to-end, including structured final answers and forensic exchange capture,
so the first useful Inception slice behaves like the other remote providers.

## Acceptance Criteria

- [x] Runtime preparation can route `Inception + mercury-2` through the OpenAI-compatible HTTP adapter without introducing a bespoke execution path [SRS-02/AC-01]. <!-- verify: cargo test -q prepare_runtime_lanes_treats_inception_as_remote_http_lane_without_local_paths, SRS-02:start:end, proof: ac-1.log-->
- [x] Mercury-2 requests and responses support the structured final-answer path expected by paddles, or fail over through the existing rendering contract without breaking turns [SRS-02/AC-02]. <!-- verify: cargo test -q inception_provider_normalizes_structured_final_answers, SRS-02:start:end, proof: ac-2.log-->
- [x] Inception request/response exchanges are captured through the existing forensic artifact path [SRS-NFR-02/AC-03]. <!-- verify: cargo test -q inception_provider_records_exact_forensic_exchange_artifacts_in_trace_replay, SRS-NFR-02:start:end, proof: ac-3.log-->
