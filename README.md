> **⛔ RC1 REJECTED BY INDEPENDENT AUDIT.**
> The tag `aeon-air-defense-rc1` (commit `d1b8414`) is an **engineering
> prototype / lab baseline**, NOT a repository-verified integration-ready
> release. Do **not** advance it through sponsor Gate 11B.
> See [`docs/verification/RC1_AUDIT_REJECTION.md`](docs/verification/RC1_AUDIT_REJECTION.md).
> Corrections proceed on a new RC2 branch; RC1 history is preserved intact.

# Aeon Air Defense Information Layer (AAMDS)

Aeon is an **equipment-neutral information, tracking, fusion, operator-awareness,
and secure-relay platform** for authorized air-object observations. It ingests,
normalizes, correlates, evaluates, displays, records, and securely relays
air-object tracking information.

> Aeon is **not** a weapon-control system. It does not select, prioritize, aim,
> guide, arm, launch, or fire weapons. See
> [`docs/architecture/SCOPE_BOUNDARY.md`](docs/architecture/SCOPE_BOUNDARY.md)
> and the mechanical scope-boundary tests under
> [`verification/scope-boundary/`](verification/scope-boundary/).

## Layout

| Path | Purpose |
| --- | --- |
| `contracts/` | Versioned canonical domain schemas (Rust types + serde). |
| `core-runtime/` | Runtime supervisor, lifecycle, health, structured logs. |
| `persistence/` | Append-only event store (SQLite / Postgres-compatible). |
| `sensor-adapter-sdk/` | Adapter trait, synthetic / replay / file / network adapters, conformance harness. |
| `normalization/` | Time, coordinate, unit, uncertainty, integrity normalization with transformation provenance. |
| `track-management/` | Deterministic uncertainty-qualified track engine (equipment-neutral). |
| `ml-correction/` | Bounded, isolated correction subsystem with registry + shadow evaluation. |
| `operator-api/` | Read-oriented HTTP API for the recognized operational picture. |
| `operator-interface/` | Minimal TypeScript/React thin client. |
| `secure-relay/` | Policy-enforced outbound relay gateway with allowlisting, signing, anti-replay. |
| `simulation/` + `replay/` | Scenario fixtures + deterministic replay runner. |
| `verification/` | Scope-boundary, determinism, and gate verification tooling. |
| `interface-control-documents/` | ICDs for adapter, observation, track, health, alert, relay schemas. |
| `deployment/` | Reproducible deployment profiles (developer, edge, fixed-site, disconnected, data-center, private-cloud). |
| `cybersecurity/` | Security scaffolding — threat model, SBOM tooling. |
| `docs/` | Architecture, operations, security, integration, model, verification docs. |

## Build

```
cargo build --workspace
cargo test  --workspace
./tools/verify.sh all
```

## Status

Additive commit sequence Phase A–N. See
[`docs/verification/KNOWN_LIMITATIONS.md`](docs/verification/KNOWN_LIMITATIONS.md)
for an honest inventory of what is implemented, what is scaffolded, and what
requires sponsor-supplied integration.

**No claim** of accreditation, deployment approval, government certification,
combat readiness, or unrestricted plug-and-play compatibility is made or
implied by this repository.
