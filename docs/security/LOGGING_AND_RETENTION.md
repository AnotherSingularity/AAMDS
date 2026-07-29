# Logging and Retention

- **Structured logs** via `tracing` + `tracing-subscriber` (JSON layer
  enabled in the operator-api binary; other binaries follow the same
  pattern).
- **No secrets, private keys, credentials, or unrestricted payload
  contents** are logged. Payload references are by digest, not by
  content.
- **Retention** for logs and audit is a policy decision configured per
  deployment profile — see `configuration_versions` in the persistence
  schema and `docs/operations/BACKUP_RESTORE.md`.
- **Audit events** are append-only and integrity-chained; retention
  cannot silently truncate history without breaking verification.
