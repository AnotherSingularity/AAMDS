# Release Evidence Index

Every release candidate MUST link to the following artifacts. Baseline
paths where the artifacts are generated:

| Artifact | Source |
|---|---|
| Source commit | `git rev-parse HEAD` |
| Build identity | `Cargo.toml` workspace version + `git describe` |
| Signed artifact manifest | `deployment/release/manifest.sha256` (signed out-of-band) |
| SBOM (CycloneDX) | `tools/verify.sh sbom` -> `**/*.cdx.json` |
| Dependency inventory | `cargo metadata --format-version=1` |
| Vulnerability report | `cargo audit --json` |
| Static-analysis report | `cargo clippy --workspace --all-targets --message-format=json` |
| Test report | `cargo test --workspace --all-targets` + `docs/verification/TEST_MATRIX.md` |
| Replay-determinism report | `cargo test -p aeon-simulation --test determinism` |
| Scope-boundary verification | `tools/verify.sh scope-boundary` |
| Configuration schemas | `core-runtime/src/config.rs` (deny_unknown_fields) |
| Database migrations | `persistence/src/migrations.rs` |
| Upgrade instructions | `docs/operations/UPGRADE.md` |
| Rollback instructions | `docs/operations/ROLLBACK.md` |
| Operator guide | `docs/operations/INSTALLATION.md`, `docs/operations/TROUBLESHOOTING.md` |
| Maintainer guide | `CONTRIBUTING.md`, `docs/models/MODEL_GOVERNANCE.md` |
| Integration guide | `interface-control-documents/` |
| Known limitations | `docs/verification/KNOWN_LIMITATIONS.md` |
| Security notices | `SECURITY.md`, `docs/security/*` |
