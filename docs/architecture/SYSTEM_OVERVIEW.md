# System Overview

Aeon is a modular, equipment-neutral information-layer platform for authorized
air-object tracking. It sits between authorized sensor sources and authorized
downstream information consumers.

```
[ authorized sensors ]
        │  (over vendor protocols)
        ▼
┌───────────────────────────────────────────────────────────────────┐
│  sensor-adapter-sdk        vendor-specific adapters               │
│      │       │       │                                            │
│      ▼       ▼       ▼                                            │
│              normalization                                        │
│                    │                                              │
│                    ▼                                              │
│               track-management (uncertainty-qualified)            │
│                    │                                              │
│      ┌─────────────┼─────────────┐                                │
│      ▼             ▼             ▼                                │
│  persistence   operator-api   secure-relay                        │
│      (append-only, audit-hashed)     (policy-enforced, signed)    │
│                    │                                              │
│                    ▼                                              │
│               ml-correction                                       │
│              (bounded, isolated, shadow-capable)                  │
└───────────────────────────────────────────────────────────────────┘
        │                                        │
        ▼                                        ▼
 [ operator UI (thin) ]              [ authorized informational peers ]
```

Every arrow crossing a component boundary is:

- a versioned schema in `contracts/`;
- provenance-carrying;
- integrity-tagged;
- deterministic under replay.

Modules under `contracts/` are the single source of truth for what may cross
a boundary. No module may communicate through undocumented back-channels.

## Runtime posture

- Fail-closed on unknown identity, invalid signatures, stale credentials,
  malformed messages, unauthorized destinations, prohibited message types,
  invalid schemas, and broken trust chains.
- Degraded modes are visible in `SystemHealth` and reflected in the operator UI.
- The ML correction subsystem may be disabled (or unavailable) without
  breaking the deterministic tracking path.

See also:

- [`SCOPE_BOUNDARY.md`](SCOPE_BOUNDARY.md)
- [`COMPONENT_MODEL.md`](COMPONENT_MODEL.md)
- [`DATA_FLOW.md`](DATA_FLOW.md)
- [`DEPLOYMENT_MODEL.md`](DEPLOYMENT_MODEL.md)
- [`FAILURE_MODEL.md`](FAILURE_MODEL.md)
