# Evidence Collection Guide

Capture the following per acceptance run and file in the sponsor's
evidence store (path recorded on `ACCEPTANCE_RESULTS_TEMPLATE.md`):

## Universal
- `SITE_SURVEY_TEMPLATE.md` — filled and signed.
- `TARGET_ENVIRONMENT_INVENTORY.md` YAML output for this site.
- `<package>/package.manifest.json` (unmodified).
- `<package>/manifest.sha256`.
- `sha256sum` of every artifact after installation.
- `docs/evidence/gate-10/*` from the source-branch CI run that
  produced the package.
- `docs/evidence/gate-11/*` from the same run.

## Per-procedure
- **Installation**: `journalctl` / OS log excerpt from install window,
  `curl` transcripts for API `/api/v1/version` and `/api/v1/health`,
  `sqlite3 aeon.sqlite ".schema"` output.
- **Upgrade**: `UPGRADE_BEFORE.json` and `UPGRADE_AFTER.json` produced
  by `upgrade.sh`, and the pre/post sqlite3-dump digests.
- **Rollback**: `ROLLBACK.json`, `sqlite3-dump` digest match against
  the pre-upgrade capture.
- **Backup / Restore**: backup file + `.manifest.json` companion,
  restore verification digest.
- **Security**: KMS access-log excerpt covering the signing operation,
  audit chain `verify_integrity` output, TLS handshake capture.
- **Network**: pcap / netflow excerpt limited to Aeon ingress/egress
  ports.

## Retention
Per sponsor governance (see
`docs/security/LOGGING_AND_RETENTION.md`).
