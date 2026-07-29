# Installation

## Prerequisites

- Rust stable (see `rust-toolchain.toml`).
- SQLite (bundled via `rusqlite`; no separate install for developer profile).
- For production profiles, a Postgres-compatible database and a KMS/HSM
  for signature material.

## From source

```
git clone <repo>
cd aeon
cargo build --release --workspace
cargo test  --workspace       # 60+ tests
tools/verify.sh all
```

## Per-profile package

```
deployment/<profile>/build.sh
# → target/deploy/<profile>/{bin,ui,config,docs,manifest.sha256}
```

## First-run smoke

1. Start the operator API:
   `target/deploy/developer/bin/aeon-operator-api`
2. Serve the UI:
   `cd target/deploy/developer/ui && python3 -m http.server 5173`
3. Open http://127.0.0.1:5173/
