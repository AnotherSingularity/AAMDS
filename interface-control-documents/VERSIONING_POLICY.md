# Versioning Policy

`SchemaVersion { name, major, minor }` on every message.

- **Major bump** (breaking): field removal, field type change,
  semantics change of an existing field, enum-variant removal or
  meaning change. Requires:
  - a new file under `contracts/src/` (or a superseded name),
  - a compatibility-matrix entry,
  - a migration in `persistence/src/migrations.rs` if the change
    affects persistence,
  - explicit ADR-style note in the commit body.
- **Minor bump** (additive): new optional field via `Known::Unavailable`,
  new enum variant that consumers can safely ignore. Runtime accepts
  minor ≤ its own.

Runtime negotiation:

- On ingest, if `msg.schema_version.major != runtime.declared.major`
  → reject with typed error, record in audit.
- If `msg.schema_version.minor > runtime.declared.minor` → accept but
  emit a `Warning` audit event so operators can see the drift.
