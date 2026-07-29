# Identity Model

Aeon distinguishes four kinds of identity:

| Identity | Represented as | Trust anchor |
|---|---|---|
| Sensor | `SensorId` (opaque string) | Adapter's `Integrity` state |
| Adapter | `AdapterId` + `adapter_version` | Runtime capability check |
| Actor (operator, service) | `ActorId` (opaque string) | Sponsor-supplied IdP (out of scope) |
| Destination (peer) | `DestinationId` + `public_key_hex` | Relay policy `DestinationPolicy` |

**No default credentials** ship with this repository. The baseline
`sign` / `verify` use a keyed SHA-256 for structural correctness; sponsor
deployments **must** substitute FIPS-validated primitives via KMS/HSM
before production use.
