# Known Limitations

This document lists **honestly** what is implemented, what is scaffolded, and
what requires sponsor-supplied integration to complete. It is normative for
the `MILITARY INTEGRATION-READY BASELINE` claim: no gate in the final report
may be marked PASS unless the item is mechanically verified.

## Implemented and mechanically verified

- Canonical domain contracts (`contracts/`): versioned Rust types with
  serde round-trip tests and the `Known<T>` unknown-state model.
- Prohibited-concept registry and JSON scanner (`contracts::prohibited`).
- Shell-based scope-boundary static scan (`tools/scope_boundary_scan.sh`),
  invoked from `verify.sh scope-boundary` and CI job `verify-scope-boundary`.
- Architecture documentation set (`docs/architecture/`).
- Root verification driver (`tools/verify.sh`) with the target list from
  section 30 of the implementation directive; missing tools produce a
  clear installation error rather than a silent skip.

## Scaffolded (compiles but not exhaustive)

- Runtime supervisor with lifecycle states and health snapshot generation.
- Persistence: SQLite append-only event store, integrity-chained audit log.
- Sensor adapter SDK trait, synthetic + replay + file adapters, conformance
  harness covering malformed / duplicate / out-of-order / clock-rollback /
  restart cases.
- Normalization: time, coordinate, unit, uncertainty, integrity with
  transformation-chain provenance.
- Track management: deterministic uncertainty-qualified engine with
  correlation, duplicate suppression, conflict handling.
- ML correction: bounded-magnitude correction pipeline with model registry,
  shadow evaluation, OOD marking, and rollback.
- Secure relay: allowlist, prohibited-content scan, anti-replay nonce,
  store-and-forward queue.
- Simulation + deterministic replay runner and scenario fixtures.

## Explicitly deferred to sponsor integration

The following require sponsor-supplied trust anchors, hardware, or
accredited environments and are therefore **not** claimed as complete:

- Vendor-specific sensor adapters. The SDK, template, and conformance
  harness are provided; individual vendor integrations are not.
- Production identity provider integration. The relay authenticates against
  a configurable trust store; enterprise IdP wiring is deployment-specific.
- Full six-profile deployment automation. Profiles are documented and
  scripts are stubbed; end-to-end reproducibility requires sponsor CI.
- Cryptographic-inventory FIPS/NIAP evidence. Only the *set* of primitives
  used is documented; formal validation is sponsor responsibility.
- Accreditation, deployment approval, government certification, combat
  readiness. **No such claim** is made by this repository.

## Repository-level constraints observed in this session

- The initial commit sequence was produced in a remote, ephemeral
  container. Reproducibility depends on the pinned toolchain versions in
  `rust-toolchain.toml` (if present) and `Cargo.lock`.
- The workspace begins as a Rust-only baseline. The TypeScript operator UI
  is minimal and consumes the operator API through the same versioned
  contracts (`contracts/`). It is not a substitute for a fully engineered
  operator workstation.
