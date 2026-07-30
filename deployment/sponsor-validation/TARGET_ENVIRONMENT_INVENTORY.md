# Target Environment Inventory

Machine-readable-style inventory the sponsor completes at each target.
Recommended format: YAML file per site, following the schema below.

```yaml
site_code: "SITE-XYZ"
host:
  hostname: ""
  os: {name: "", version: "", kernel: ""}
  cpu:  {model: "", cores: 0}
  memory_mb: 0
  disk:
    - {path: "/var/lib/aeon", size_mb: 0, medium: ""}
network:
  interfaces:
    - {name: "", ipv4: "", vlan: ""}
  ingress:
    - {label: "sensor-a", protocol: "", endpoint: ""}
  egress:
    - {label: "peer-a",   destination: "", policy: ""}
identity:
  idp: {name: "", url: "", protocol: ""}
  role_bindings:
    - {aeon_role: "operator", idp_group: ""}
crypto:
  kms: {vendor: "", model: "", key_ids: []}
  signing_key_id: ""
  encryption_at_rest: {method: "", key_ref: ""}
time:
  primary: ""
  secondary: ""
  measured_drift_ms: 0
package:
  profile: ""
  version: ""
  commit: ""
  manifest_sha256: ""
```
