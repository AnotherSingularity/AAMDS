# Conformance Testing

Aeon ships four layers of conformance:

1. **Contract**: `cargo test -p aeon-contracts` — 12 tests including
   serde round-trip and property tests.
2. **Adapter**: `cargo test -p aeon-sensor-adapter-sdk` — 5 conformance
   cases exercised against the synthetic and replay adapters. A new
   adapter is expected to run the same harness against itself.
3. **Pipeline determinism**:
   `cargo test -p aeon-simulation --test determinism` — proves that
   identical inputs yield identical trace digests.
4. **Scope boundary**: `tools/verify.sh scope-boundary` — static scan
   over the workspace.

Full sweep: `tools/verify.sh all`.

## Authoring a new adapter (integration-readiness check)

- [ ] Read `SENSOR_ADAPTER_ICD.md`.
- [ ] Implement `SensorAdapter` for your source.
- [ ] Populate `AdapterCapability` truthfully.
- [ ] Add unit tests covering: connection loss, malformed input,
      out-of-order events, unsupported schema, clock rollback,
      restart recovery.
- [ ] Run the shared conformance harness cases from
      `sensor-adapter-sdk/src/conformance.rs`.
- [ ] Confirm `tools/verify.sh scope-boundary` still passes.
- [ ] Add an entry to your adapter's crate `README.md` linking back
      to this ICD.
