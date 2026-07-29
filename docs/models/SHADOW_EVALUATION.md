# Shadow Evaluation

`ml-correction::CorrectionRuntime::shadow_evaluate` produces a
`ShadowComparison` for each input, containing the champion's applied
correction and the challenger's would-be correction. The challenger's
`applied_confidence` is always `0.0` — it never affects operational
state.

Comparisons are persisted so approvers can review distribution shift
and correction disagreement before promoting the challenger.

Promotion rule (baseline suggestion; sponsor policy will refine):

- ≥ 10 000 shadow comparisons
- 95th-percentile |Δ| below policy threshold
- No `Rejected` challenger events in the last 24h
