# Traceability Matrix

Directive requirement → source location + test.

| Directive | Location | Test |
|---|---|---|
| 2. Scope boundary registry | `contracts/src/prohibited.rs` | `contracts::prohibited::tests` |
| 2. Compile-time relay allowlist | `secure-relay/src/allowlist.rs` | `allowlist::tests::allowlist_size_is_exactly_four` |
| 2. Runtime outbound content scan | `secure-relay/src/gateway.rs::submit` | `gateway::tests::prohibited_content_is_rejected` |
| 2. Static source scan | `tools/scope_boundary_scan.sh` | `verify-scope-boundary` CI |
| 6.2 Determinism | `simulation/src/determinism.rs` | `simulation/tests/determinism.rs` |
| 6.3 Honest uncertainty | `contracts/src/unknown.rs` | `unknown::tests`, `property::known_roundtrip_preserves_variant` |
| 6.4 Provenance | `contracts/src/provenance.rs` | populated by every pipeline stage (see `simulation/src/pipeline.rs`) |
| 6.5 Fail-closed | `core-runtime/src/config.rs` production validation | `config::tests::production_requires_outbound_scan_and_signed_sources` |
| 7.x Canonical model | `contracts/src/*.rs` | `contracts` test suite |
| 8. Runtime lifecycle | `core-runtime/src/lifecycle.rs` | `lifecycle::tests` |
| 9. Adapter SDK + conformance | `sensor-adapter-sdk/` | `conformance::tests` |
| 10. Normalization | `normalization/src/lib.rs` | `normalization::tests` |
| 11. Track management | `track-management/src/lib.rs` | `track-management::tests` |
| 12. Bounded ML correction | `ml-correction/src/lib.rs` | `ml-correction::tests` |
| 13. Operator API | `operator-api/src/routes.rs` | `operator-api/tests/routes.rs` |
| 15. Secure relay | `secure-relay/src/gateway.rs` | `gateway::tests` |
| 16. Persistence + audit | `persistence/src/store.rs` | `store::tests` + `persistence/tests/adversarial.rs` |
| 17. Simulation + replay | `simulation/` | `simulation/tests/*` |
| 18. Security architecture | `docs/security/*.md` | doc validation |
| 19. Configuration | `core-runtime/src/config.rs` | `config::tests` |
| 20. Deployment profiles | `deployment/` | per-profile `build.sh` |
| 21. ICDs | `interface-control-documents/` | doc validation |
| 22. Observability | `operator-api/src/main.rs` (`tracing_subscriber::fmt().json()`) | manual |
| 23. Testing strategy | `docs/verification/TEST_MATRIX.md` | `verify.sh all` |
| 24. CI | `.github/workflows/ci.yml` | GitHub CI |
| 25. Release engineering | `docs/evidence/RELEASE_EVIDENCE_INDEX.md` | manual |
| 30. Verification driver | `tools/verify.sh` | `tools/verify.sh all` |
