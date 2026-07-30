# Acceptance Results Template

One completed copy per acceptance run, filed with sponsor evidence.

## Identification
- Site code:
- Package version:
- Source commit:
- Package manifest sha256:
- Date + timezone:
- Change-control ticket:

## Team
- Installer:
- Observer:
- Security-approver:
- Network engineer:

## Results

| Procedure | Reference doc | Result | Evidence link | Notes |
|---|---|---|---|---|
| Pre-installation | `PREINSTALLATION_CHECKLIST.md` | PASS / FAIL | | |
| Site survey | `SITE_SURVEY_TEMPLATE.md` | PASS / FAIL | | |
| Installation | `INSTALLATION_ACCEPTANCE_PROCEDURE.md` | PASS / FAIL | | |
| Upgrade | `UPGRADE_ACCEPTANCE_PROCEDURE.md` | PASS / FAIL / N/A | | |
| Rollback | `ROLLBACK_ACCEPTANCE_PROCEDURE.md` | PASS / FAIL / N/A | | |
| Backup + restore | `BACKUP_RESTORE_ACCEPTANCE_PROCEDURE.md` | PASS / FAIL | | |
| Security | `SECURITY_VALIDATION_CHECKLIST.md` | PASS / FAIL | | |
| Network integration | `NETWORK_INTEGRATION_CHECKLIST.md` | PASS / FAIL | | |
| Scope boundary | `docs/architecture/SCOPE_BOUNDARY.md` scan on deployed commit | PASS / FAIL | | |

## Overall verdict

- [ ] **ACCEPTED** — all mandatory procedures PASS on the target
      environment; sponsor authorizes Aeon at this site.
- [ ] **CONDITIONALLY ACCEPTED** — one or more procedures returned
      FAIL / N/A but sponsor authorizes with documented compensating
      controls listed below.
- [ ] **REJECTED** — one or more mandatory procedures FAILED.

## Compensating controls (if conditionally accepted)

Enumerate.

## Signatures
- Installer:
- Observer:
- Security-approver:
- Sponsor authorizing officer:
