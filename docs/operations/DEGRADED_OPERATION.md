# Degraded Operation

See `docs/architecture/FAILURE_MODEL.md` for the full table. Highlights:

- Adapter loss → track freshness ages, no new correlations from that
  adapter; operator UI shows the adapter as disconnected.
- Storage read-only → runtime enters `Degraded`; new events queue up to
  the bounded backpressure limit; audit continues via WAL.
- Model unavailable → correction subsystem disables that model; core
  tracking is deterministic and continues.
- Relay destination down → envelopes queue; beyond depth policy they
  dead-letter and an operator alert is raised.
- Time source degraded → `TimeQuality::Drifting`; observations from
  affected sources become suspect and may be rejected by policy.

Degraded state is always visible in `SystemHealth.degraded_capabilities`
and in the operator UI header.
