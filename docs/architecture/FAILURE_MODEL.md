# Failure Model

Aeon degrades **visibly** rather than silently.

| Failure | Behaviour | Signal |
|---|---|---|
| Adapter connection loss | Adapter reports `AdapterHealth.connected=false`; contributing tracks age; correlations no longer accept observations from that adapter | `SystemHealth`, `Alert(AdapterFailure)` |
| Sensor dropout | Sensor feed rate drops to zero; freshness state on affected tracks transitions Fresh → Aging → Stale | `Alert(SensorDropout)` |
| Clock drift beyond policy | Time quality becomes `Drifting`; correlation refuses observations from that source | `Alert(ClockDrift)` |
| Storage read-only | Runtime enters `Degraded`; new observations queue in bounded backpressure; audit continues via WAL | `Alert(StorageDegraded)` |
| Relay destination failure | Envelopes queue up to policy limit, then dead-letter | `Alert(RelayDegraded)` |
| Integrity failure | Envelope / observation rejected; audit event `IntegrityViolation` | `Alert(IntegrityViolation)` |
| Invalid configuration | Runtime refuses to promote from `ValidatingConfiguration`; last-good stays active | `Alert(ConfigurationInvalid)` |
| Model unavailable | Correction subsystem disables that model; core tracking continues without correction | `Alert(ModelUnavailable)` |
| Model input OOD | Correction marked `Untrusted`, core tracking uses raw path | `Alert(ModelOutOfDistribution)` |
| Prohibited outbound content | Relay rejects, audit `ScopeBoundaryViolationAttempt` | `Alert(ScopeBoundaryViolationAttempt)` |

**Failing closed** is preferred to guessing. There is no path in the code where
"missing" silently becomes "nominal".
