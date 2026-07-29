# Deployment Profiles

Six profiles per directive section 20. Each profile has:

- a `README.md` describing scope and how the profile is intended to be used;
- a `build.sh` script that packages the profile from a clean checkout;
- profile-specific configuration under `config/`;
- a `manifest.txt` template listing artifacts + digests for the release
  evidence index.

**No profile claims accreditation or deployment approval.** These are
engineering targets that a sponsor accreditation process can validate.

| Profile | Directory | Runtime | Persistence | Relay |
|---|---|---|---|---|
| Developer | `developer/` | in-process | SQLite | simulator |
| Edge | `edge/` | single-node | SQLite | store-and-forward |
| Fixed-site | `fixed-site/` | redundant | Postgres-compatible | HA option |
| Disconnected | `disconnected/` | single-node | SQLite | none |
| Data centre | `data-center/` | multi-instance | Postgres-compatible | policy-controlled |
| Approved private cloud | `private-cloud/` | multi-instance | Postgres-compatible | policy-controlled |

The reference build scripts in this baseline are `cargo build --release`
wrappers plus profile-specific config assembly. Sponsors are expected to
substitute their own container base image, package format, and signing
key before production rollout.
