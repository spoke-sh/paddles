---
# system-managed
id: VFMCtJc0q
status: icebox
created_at: 2026-03-30T06:30:47
updated_at: 2026-03-30T06:30:47
# authored
title: Mistral RS Inference Adapter
type: feat
operator-signal:
scope: VFMC5HaQY/VFMCRb7SD
index: 1
---

# Mistral RS Inference Adapter

## Summary

Create `LocalInferenceAdapter` in `src/infrastructure/adapters/mistralrs_provider.rs` using mistral.rs as the inference backend. Implements `SynthesizerEngine` and `RecursivePlanner` domain ports, supporting multiple model architectures and GGUF quantization. Coexists with existing `SiftAgentAdapter` as `--provider local` vs `--provider sift`.

## Acceptance Criteria

- [ ] LocalInferenceAdapter loads models via mistral.rs MistralRs struct [SRS-01] <!-- verify: test, SRS-01:start:end -->
- [ ] LocalInferenceAdapter implements SynthesizerEngine for text generation [SRS-02] <!-- verify: test, SRS-02:start:end -->
- [ ] LocalInferenceAdapter implements RecursivePlanner via structured JSON parsing [SRS-03] <!-- verify: test, SRS-03:start:end -->
- [ ] --provider local --model <id> routes to LocalInferenceAdapter [SRS-04] <!-- verify: manual, SRS-04:start:end -->
- [ ] GGUF quantized models load and run via mistral.rs [SRS-05] <!-- verify: manual, SRS-05:start:end -->
- [ ] Device selection (CUDA/CPU) uses mistral.rs built-in detection [SRS-06] <!-- verify: manual, SRS-06:start:end -->
- [ ] Existing SiftAgentAdapter continues to work as --provider sift [SRS-NFR-01] <!-- verify: test, SRS-NFR-01:start:end -->
