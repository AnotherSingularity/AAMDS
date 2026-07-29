# Sensor Adapter ICD

## Purpose
Integrate a new sensor source into Aeon by implementing `SensorAdapter`.

## Scope
Applies to any adapter shipped inside `sensor-adapter-sdk/src/adapters/`
or maintained out-of-tree. Vendor-specific transport code lives in the
adapter; the SDK trait and conformance harness are the compatibility
contract.

## Inputs
Vendor-native messages (adapter-defined).

## Outputs
`aeon_contracts::observation::RawObservation` values, one per received
measurement.

## Required fields on RawObservation
See `OBSERVATION_SCHEMA.md` — every field on `RawObservation` is
required. Adapters that cannot supply a field MUST populate it with
the appropriate `Known` variant (`Unknown`, `Unavailable`, etc.), never
a silent default.

## Optional fields
None. If your source cannot supply the value, use the appropriate
`Known` variant.

## Units
- Position: WGS84 geodetic degrees + metres, ECEF metres, or
  range-bearing-elevation in metres / degrees. Vendor-native units
  MUST be recorded in the raw payload; conversion happens in
  normalization.
- Time: RFC3339 UTC (source and receive).
- Uncertainty: 1σ metres (position), m/s (velocity).

## Coordinate conventions
Use `CoordinateReference::Wgs84Geodetic` where possible. If the source
frame is unknown, use `CoordinateReference::Unknown` — normalization
will reject it with a typed error, which is the correct fail-closed
behaviour.

## Time conventions
Adapters MUST set `TimeQuality` to the actual quality state
(`Disciplined`, `LocalMonotonic`, `Drifting`, `ReceiveTimeSubstituted`,
`Unavailable`). Never claim `Disciplined` for a source that is not
timed against a trusted reference.

## Error behaviour
Return typed `AdapterError` variants. Malformed payloads,
unsupported schemas, clock rollback, and duplicate sequence numbers
MUST NOT be silently discarded — they are audited via the
persistence layer's `rejected_observations` table.

## Security behaviour
Adapter processes SHOULD run in a separate OS process from the core
runtime with least-privilege permissions. `Integrity` state on each
observation propagates downstream through `Integrity::Derived`.

## Version negotiation
`RawObservation.schema_version` MUST equal
`aeon_contracts::version::observation_schema()`. A mismatched major
is rejected. A newer minor is accepted with an audit note.

## Backward compatibility
Contracts guarantee `major.minor` stability within a `major`. Bumping
the major requires an entry in `COMPATIBILITY_MATRIX.md`.

## Example
See `sensor-adapter-sdk/src/adapters/synthetic.rs` for a complete
in-tree adapter that produces bit-identical output across runs.

## Conformance
`SensorAdapter` conformance is asserted by
`sensor-adapter-sdk/src/conformance.rs`. Every adapter MUST pass the
five directive-mandated cases (pre-connect refusal, malformed input,
duplicate sequence, clock rollback, restart recovery) plus any
adapter-specific harness cases.
