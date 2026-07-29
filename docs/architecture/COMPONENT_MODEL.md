# Component Model

| Component | Owner | Responsibilities | Not permitted |
|---|---|---|---|
| `contracts` | domain | Versioned Rust types + JSON schemas | Any business logic |
| `core-runtime` | platform | Lifecycle, health, structured logs, config validation | Any sensor or model logic |
| `persistence` | platform | Append-only event store, migrations, integrity | Overwriting historical events |
| `sensor-adapter-sdk` | integration | Adapter trait, reference adapters, conformance harness | Vendor-specific business logic in core |
| `normalization` | domain | Canonical time, coords, units, uncertainty, integrity | Silent defaulting of missing values |
| `track-management` | domain | Correlation, uncertainty propagation, conflict handling | Engagement recommendations |
| `ml-correction` | domain | Bounded, isolated correction, shadow evaluation | Any prohibited concept (see `SCOPE_BOUNDARY.md`) |
| `operator-api` | interface | Read-oriented HTTP API | Writes beyond ack / annotate / activation |
| `operator-interface` | interface | Thin client | Fusion or security logic |
| `secure-relay` | boundary | Allowlist-guarded outbound relay | Anything not in `RelayMessageKind` |
| `simulation` / `replay` | verification | Deterministic scenarios and replay | Live external transmission |

Every component depends *only* on `contracts` for cross-boundary data. No
runtime component depends on another runtime component's private modules.
