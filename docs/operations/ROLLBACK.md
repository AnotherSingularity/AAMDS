# Rollback

Two independent rollback paths exist.

## Runtime rollback (binaries + schema)

1. Stop the current runtime (`ShuttingDown → Uninitialized`).
2. Restore the pre-upgrade persistence snapshot (see BACKUP_RESTORE.md).
3. Re-deploy the prior release's binaries and configuration.
4. Start the runtime; readiness requires the pre-existing
   `configuration_version` digest to validate.

## Model rollback (ML correction)

1. Identify the last-known-good model:
   `select id, name, version, state from model registry`.
2. `ModelRegistry::rollback_active_to(id)` — transitions the previously-
   `Deprecated` model back to `Approved` then `Active`, demoting the
   current champion to `Deprecated`.
3. Verify by inspecting the operator API `/api/v1/health` `active_model_ids`.
