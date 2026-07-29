# Test Matrix

Numbers reflect the baseline commit sequence. All tests are green.

| Crate / target | Unit | Property | Integration | Adversarial | Total |
|---|---|---|---|---|---|
| `aeon-contracts` | 6 | 6 | – | – | 12 |
| `aeon-core-runtime` | 12 | – | – | – | 12 |
| `aeon-persistence` | 4 | – | – | 2 | 6 |
| `aeon-sensor-adapter-sdk` | 5 | – | – | – | 5 |
| `aeon-normalization` | 6 | – | – | – | 6 |
| `aeon-track-management` | 9 | – | – | – | 9 |
| `aeon-ml-correction` | 7 | – | – | – | 7 |
| `aeon-secure-relay` | 15 | – | – | – | 15 |
| `aeon-operator-api` | – | – | 5 | – | 5 |
| `aeon-simulation` | – | – | 4 | – | 4 |
| **Total** | **64** | **6** | **9** | **2** | **81** |

Additional mechanical gates:

- Scope-boundary static scan: PASS (shell scanner over the workspace).
- Deterministic replay: PASS (two runs of the same scenario → equal
  trace digest; different scenarios → different digests).
- Documentation presence: PASS (`tools/verify.sh docs`).
