# edge profile

Reference deployment package assembly. See `deployment/README.md` for
the profile taxonomy.

Build: `deployment/edge/build.sh` → `target/deploy/edge/`
Config:  `deployment/edge/config/runtime.json`

Baseline scope: this profile produces a signed manifest of the release
binaries + UI + docs. Sponsor deployments are expected to substitute
their own container base image, orchestrator (systemd / k8s / nomad),
identity provider, KMS/HSM, and observability sinks.

**No claim** of accreditation, cloud approval, or platform certification.
