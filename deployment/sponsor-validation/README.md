# Sponsor Validation Package

This directory contains everything an authorized sponsor's engineering /
security / operations personnel need to run Aeon's target-environment
acceptance procedures without inventing steps. Nothing in this package
replaces the repository's mechanical verification (Gate 10, Gate 11A);
it complements it with the sponsor-owned Gate 11B evidence.

## Contents

| File | Purpose |
|---|---|
| `PREINSTALLATION_CHECKLIST.md` | Preconditions the target environment must satisfy before Aeon can be installed. |
| `SITE_SURVEY_TEMPLATE.md` | Fill-in template for hardware, network, identity, and time infrastructure. |
| `TARGET_ENVIRONMENT_INVENTORY.md` | Machine-readable-style inventory of what the sponsor's target actually contains. |
| `INSTALLATION_ACCEPTANCE_PROCEDURE.md` | Step-by-step installation acceptance on the target. |
| `UPGRADE_ACCEPTANCE_PROCEDURE.md` | Upgrade acceptance in the target environment. |
| `ROLLBACK_ACCEPTANCE_PROCEDURE.md` | Rollback acceptance under the sponsor's supported conditions. |
| `BACKUP_RESTORE_ACCEPTANCE_PROCEDURE.md` | Backup + restore acceptance including data-loss RPO/RTO validation. |
| `SECURITY_VALIDATION_CHECKLIST.md` | KMS/HSM integration, credential rotation, encryption-at-rest, audit-log export. |
| `NETWORK_INTEGRATION_CHECKLIST.md` | Ingress adapter, egress relay, IdP, observability integrations. |
| `EVIDENCE_COLLECTION_GUIDE.md` | What to capture during acceptance, in what format, and where to file it. |
| `ACCEPTANCE_RESULTS_TEMPLATE.md` | Fill-in template for the final acceptance sign-off. |

## Scope

This package does **not** include weapon-integration procedures or
firing / engagement / launch / guidance validation of any kind. Aeon is
an information-layer platform; the acceptance checks here validate that
its information-layer capability lands cleanly in the target environment.
