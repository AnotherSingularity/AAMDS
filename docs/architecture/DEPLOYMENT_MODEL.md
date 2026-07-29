# Deployment Model

Aeon ships as a Cargo workspace producing a small set of native binaries and a
thin TypeScript operator UI. Deployment is per-profile — see
[`../../deployment/`](../../deployment/) for reproducible assets.

| Profile | Runtime | Persistence | Relay | Notes |
|---|---|---|---|---|
| Developer | in-process | SQLite | simulator | fastest inner loop |
| Edge | single-node | SQLite | store-and-forward | disconnected-tolerant |
| Fixed-site | redundant | Postgres-compatible | HA option | central monitoring |
| Disconnected | single-node | SQLite | none (offline) | manual export/import |
| Data-centre | multi-instance | Postgres-compatible | policy | central identity + observability |
| Approved private cloud | multi-instance | Postgres-compatible | policy | infra-as-code, signed artifacts |

**No claim** of accreditation, cloud approval, or platform certification is
implied. The profiles are engineering targets a sponsor's accreditation
process can validate against.
