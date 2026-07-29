# Compatibility Matrix

| Schema | Baseline major.minor | Introduced in | Notes |
|---|---|---|---|
| observation | 1.0 | Phase B | RawObservation |
| normalized | 1.0 | Phase B | NormalizedObservation |
| track | 1.0 | Phase B | Track |
| track_update | 1.0 | Phase B | TrackUpdate |
| health | 1.0 | Phase B | SystemHealth |
| alert | 1.0 | Phase B | Alert |
| relay_envelope | 1.0 | Phase B | RelayEnvelope |
| audit | 1.0 | Phase B | AuditEvent |

## Runtime compatibility

| Runtime version | Contracts (major) | Notes |
|---|---|---|
| 0.1.x | 1 | Baseline |

Any change to any row above requires an accompanying entry in
`VERSIONING_POLICY.md` and a migration where persistence is affected.
