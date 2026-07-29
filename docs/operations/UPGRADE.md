# Upgrade

Aeon uses in-order migrations (`persistence/src/migrations.rs`) with a
`schema_migrations` bookkeeping table. Upgrading is:

1. Stop the runtime cleanly (`ShuttingDown → Uninitialized`).
2. Back up the persistence file (`docs/operations/BACKUP_RESTORE.md`).
3. Deploy the new binaries (per-profile `build.sh` output).
4. Start the runtime; migrations run under the initial connection.
5. Confirm `EventStore::verify_integrity` returns clean and the operator
   API `/api/v1/version` reports the new build id.

No downgrade path is supported through the schema — rollback is a
restore from the pre-upgrade backup + re-deploy of the prior binaries.
