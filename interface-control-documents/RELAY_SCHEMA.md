# RELAY_SCHEMA

## Purpose
Wire contract for `RelayEnvelope`.

## Canonical Rust definition
See `../contracts/src/relay.rs`.

## Required fields
Every field on `RelayEnvelope` is required (Rust struct fields with no
`Option`). Fields that may be absent are represented by the
`Known<T>` variant.

## Optional fields
None (see above).

## Units
Position: WGS84 geodetic degrees + metres (or ECEF metres, or RBE).
Velocity: m/s. Time: RFC3339 UTC. Uncertainty: 1σ.

## Coordinate conventions
Canonical output is WGS84 geodetic. See `OVERVIEW.md`.

## Time conventions
Canonical time = source timestamp (UTC) unless `TimeQuality` says
otherwise.

## Error behaviour
Rejection returns a typed error; the rejected value is persisted (see
`persistence/src/store.rs::record_rejected_observation` and the
audit log).

## Security behaviour
See `SECURITY_REQUIREMENTS.md`. The relay layer additionally scans
outbound payloads for prohibited concepts.

## Version negotiation
`schema_version.major` must equal the runtime's declared major;
higher minor is accepted.

## Backward compatibility
Additive minor bumps only. Breaking changes require a major bump and
a compatibility-matrix entry.

## Example payloads
Runnable synthetic examples are produced by
`aeon-simulation::scenarios::single_clean_track`. Serialize any
resulting value with `serde_json::to_string_pretty` to obtain a
canonical example.

## Conformance
- `aeon-contracts` tests: 12 including serde round-trip and property
  tests over `Known<T>`, uncertainty, and confidence.
- CI job `unit-tests` runs these on every push.
