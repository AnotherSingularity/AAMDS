# Rollback Acceptance Procedure

Run on the sponsor's target environment. Prerequisites: a completed
installation acceptance (`INSTALLATION_ACCEPTANCE_PROCEDURE.md`).

## 1. Preconditions
- [ ] Currently installed version + commit recorded.
- [ ] rollback-target package is available with a validated manifest.

## 2. Execute
Run the shipped `rollback.sh` per its `--help` output, using the sponsor's
maintenance window and change-control ticket.

## 3. Observable checks
- Post-rollback `/VERSION` matches the intended target.
- Post-rollback `sha256sum -c /manifest.sha256` PASSES.
- Post-rollback operator API `/api/v1/version` reports the new build id.
- Post-rollback `EventStore::verify_integrity` PASSES (via sponsor's
  admin tooling).
- Post-rollback deterministic replay of the standard sponsor scenario
  fixture produces the expected trace digest.

## 4. Data preservation
- For upgrade + rollback: sqlite3 .dump digest MUST be preserved
  across the operation (or, for a schema-changing migration,
  RESTORE from the pre-operation backup MUST succeed and produce
  the expected digest).

## 5. Scope-boundary re-check
- No relay output during or after the rollback window may contain a
  prohibited concept (see `contracts/src/prohibited.rs`).

## 6. Sign-off
- Operator:
- Observer:
- Security-approver:
- Result (PASS / FAIL):
- Attached evidence:
