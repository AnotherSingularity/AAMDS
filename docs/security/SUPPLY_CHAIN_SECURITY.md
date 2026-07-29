# Supply-Chain Security

- **Pinned dependencies** in `Cargo.lock` (committed).
- **`cargo audit`** run by `tools/verify.sh security` (advisory only in
  CI until sponsor policy dispositions findings; failing PRs may still
  merge iff `docs/evidence/dependency-dispositions.md` justifies each
  finding).
- **SBOM** via `cargo cyclonedx` (`tools/verify.sh sbom`); the JSON
  artifact is uploaded from CI.
- **Secret scanning** via `gitleaks` (`cybersecurity/gitleaks.toml`
  policy).
- **Static analysis**: `cargo clippy --workspace --all-targets -- -D warnings`
  (`tools/verify.sh lint`).
- **Reproducible builds**: `rust-toolchain.toml` pins compiler version
  (create in the sponsor's release branch); release workflow uses
  immutable commit references.
- **Signed release artifacts**: baseline emits a manifest listing SHA-256
  digests; signing is a sponsor operation with their release key.
