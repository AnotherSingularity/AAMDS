# Incident Response

Response is triggered by an alert of category:

- `IntegrityViolation`
- `ScopeBoundaryViolationAttempt`
- `ConfigurationInvalid`
- persistent `AdapterFailure` / `RelayDegraded` / `StorageDegraded`

Playbook (baseline):

1. **Capture** — pull the last N minutes of audit events (
   `EventStore` walk + `verify_integrity`) and the current
   `SystemHealth` snapshot.
2. **Contain** — if a relay destination is implicated, remove it from
   the active policy (activate a new configuration version).
3. **Preserve** — take a filesystem snapshot of the SQLite store; the
   append-only design makes this safe.
4. **Analyse** — replay the captured window in a sandbox instance in
   `replay_mode`; compare trace digests against the last known good run.
5. **Recover** — activate the last known-good configuration; if a model
   is implicated, roll back via `ModelRegistry::rollback_active_to`.
6. **Report** — sponsor process governs external notification.
