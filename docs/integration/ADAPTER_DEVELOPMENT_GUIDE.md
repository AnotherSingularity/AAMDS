# Adapter Development Guide

See `../../interface-control-documents/SENSOR_ADAPTER_ICD.md` for the
contract; see `sensor-adapter-sdk/src/adapters/synthetic.rs` for a
worked example.

Steps:

1. Add a crate (or a module) that depends on `aeon-sensor-adapter-sdk`
   and `aeon-contracts`.
2. Implement `SensorAdapter` — validate configuration, connect,
   translate vendor messages into `RawObservation` values, expose a
   diagnostic snapshot, support clean shutdown.
3. Add tests using the conformance harness.
4. Update `docs/verification/TEST_MATRIX.md`.

**Never** import from `secure-relay`, `operator-api`, or `persistence`
in an adapter — adapters live at the untrusted boundary.
