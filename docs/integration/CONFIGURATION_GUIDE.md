# Configuration Guide

`RuntimeConfig` is the typed root of runtime configuration.
`deny_unknown_fields` rejects unrecognised keys — this catches config
drift and never silently accepts secrets under an unexpected name.

Fields:

| Field | Meaning | Fail-closed default |
|---|---|---|
| `runtime_id` | Identity of this runtime instance | required |
| `build_id` | Human-friendly build tag | required |
| `max_ingest_queue` | Bounded inbound queue depth | > 0 |
| `max_relay_queue` | Bounded outbound queue depth | > 0 |
| `freshness_seconds` | Track freshness policy | > 0 |
| `clock_drift_tolerance_ms` | Time-quality trip threshold | ≥ 0 |
| `scope_boundary_scan_outbound` | Runtime outbound content scan | `true` in production |
| `allow_unsigned_sources` | Permit unsigned inbound sources | `false` in production |

Secrets are not stored in `RuntimeConfig` — see
`docs/security/IDENTITY_MODEL.md`.
