# private-cloud profile

Reference deployment package assembly. See `deployment/README.md` for
the profile taxonomy.

Build: `deployment/private-cloud/build.sh` → `target/deploy/private-cloud/`
Config:  `deployment/private-cloud/config/runtime.json`

Baseline scope: this profile produces a signed manifest of the release
binaries + UI + docs. Sponsor deployments are expected to substitute
their own container base image, orchestrator (systemd / k8s / nomad),
identity provider, KMS/HSM, and observability sinks.

**No claim** of accreditation, cloud approval, or platform certification.
