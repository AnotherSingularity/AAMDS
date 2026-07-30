# Site Survey Template

Fill in for every target site. Attach evidence in `EVIDENCE_COLLECTION_GUIDE.md`.

## Site identification
- Site name / code:
- Location classification:
- Point of contact (name, role, contact):

## Hardware
- Host CPU model / cores:
- RAM (MiB):
- Disk (MiB) + medium:
- Redundant power:
- Redundant network:

## Operating system
- Distribution + release:
- Kernel:
- SELinux / AppArmor state:
- init system:

## Time source
- NTP / PTP servers:
- GNSS discipline available (Y/N):
- Expected drift budget:

## Network
- Ingress adapter endpoints:
- Egress relay destinations + policy:
- Firewall / diode / cross-domain configuration:
- Observability endpoints:

## Identity
- Sponsor IdP (name, URL, protocol):
- Role mapping to Aeon roles (viewer / operator / maintainer / relay-service):
- Credential rotation policy:

## Cryptographic material
- KMS / HSM (vendor, model, key identifiers):
- Signing key policy:
- Encryption-at-rest policy:

## Deployment package selected
- Profile (developer / edge / fixed-site / disconnected / data-center / private-cloud):
- Version:
- Commit:
- Manifest sha256:
