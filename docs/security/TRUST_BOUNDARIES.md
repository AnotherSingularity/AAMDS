# Trust Boundaries

```
        untrusted           semi-trusted              trusted
 ┌──────────────────┐   ┌────────────────────┐   ┌────────────────┐
 │  sensors / feeds │──▶│  adapters (per-vend)│──▶│  aeon runtime  │
 └──────────────────┘   └────────────────────┘   │  contracts     │
                                                 │  normalization │
                                                 │  track-mgmt    │
                                                 │  persistence   │
                                                 └───────┬────────┘
                                                         │
                                                    ┌────▼────┐
                              ┌──────────────────┐  │ secure- │  ┌────────────────┐
                              │ operator UI (thin│◀─│  relay  │─▶│ authorized peer│
                              │  client)         │  │ gateway │  │ (informational)│
                              └──────────────────┘  └─────────┘  └────────────────┘
```

- The **adapter boundary** is the primary attack surface. Every value
  crossing it is typed and validated; a rejected observation is persisted
  with reason.
- The **relay boundary** is the only path values leave the runtime toward
  external peers. The compile-time allowlist and runtime prohibited-content
  scan enforce the scope boundary.
- The **operator UI** is a thin client and never sees the private key
  material used for outbound signing.
