# Access Control Matrix

Baseline roles (sponsor deployments MAY narrow or extend this set):

| Role | GET tracks | GET health | GET alerts | Ack alert | Activate config | Activate model | Trigger replay | Push relay |
|---|---|---|---|---|---|---|---|---|
| viewer | ✔ | ✔ | ✔ | – | – | – | – | – |
| operator | ✔ | ✔ | ✔ | ✔ | – | – | ✔ (read-only replay) | – |
| maintainer | ✔ | ✔ | ✔ | ✔ | ✔ | ✔ (through approval) | ✔ | – |
| relay-service | – | ✔ (own destination) | – | – | – | – | – | ✔ (informational only) |

**No role** — including maintainer — grants access to a weapon-control,
firing-solution, launch, guidance, engagement, or aimpoint endpoint.
No such endpoint exists in the system.

Enforcement of these roles is out-of-scope for the baseline HTTP layer;
sponsor deployments must front the operator API with an
authenticating reverse proxy or add role-guard middleware.
