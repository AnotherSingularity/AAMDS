# Aeon ICD Set — Overview

Purpose: allow a qualified integrator to author a sensor adapter, a
downstream relay consumer, or an operator-API client using only the
information in this directory plus the canonical Rust types under
`../contracts/`.

Each ICD in this directory defines: Purpose, Scope, Inputs, Outputs,
Required fields, Optional fields, Units, Coordinate conventions,
Time conventions, Error behaviour, Security behaviour, Version
negotiation, Backward compatibility, Example payloads, Conformance
requirements.

Versioning: schema versions are recorded on every message via
`SchemaVersion { name, major, minor }`. Aeon accepts messages whose
`major` matches the runtime's declared major and whose `minor` is `<=`
runtime minor. See `VERSIONING_POLICY.md`.

Files:

- `SENSOR_ADAPTER_ICD.md` — implementing `SensorAdapter`.
- `OBSERVATION_SCHEMA.md` — `RawObservation` + `NormalizedObservation`.
- `TRACK_SCHEMA.md` — `Track`, `TrackUpdate`.
- `HEALTH_SCHEMA.md` — `SystemHealth`.
- `ALERT_SCHEMA.md` — `Alert`.
- `RELAY_SCHEMA.md` — `RelayEnvelope`.
- `SECURITY_REQUIREMENTS.md`
- `VERSIONING_POLICY.md`
- `COMPATIBILITY_MATRIX.md`
- `CONFORMANCE_TESTING.md`
