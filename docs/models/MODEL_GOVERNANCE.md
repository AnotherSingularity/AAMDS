# Model Governance

The ML correction subsystem operates under a lifecycle:

`DRAFT → VALIDATING → SHADOW → APPROVED → ACTIVE → DEPRECATED → REVOKED`

Only `ACTIVE` models influence operational corrections. `SHADOW`
produces comparison evidence without touching operational state.
`REVOKED` is terminal — a revoked model cannot be revived.

Every artifact carries: id, name, version, feature schema, output
schema, calibration cap (upper bound on confidence contribution), max
correction magnitude (bound on |Δ|), known limitations, artifact
digest, signature, state.

Approval flow:

1. Train and register (`DRAFT`).
2. Promote to `VALIDATING` after unit / offline evaluation passes.
3. Promote to `SHADOW`; run alongside champion, collect comparisons.
4. Promote to `APPROVED` after shadow comparisons meet policy.
5. Promote to `ACTIVE`; the previous champion is auto-demoted to
   `DEPRECATED`.
6. Roll back by re-approving + re-activating the deprecated model.
7. `REVOKE` if a model is unsafe.
