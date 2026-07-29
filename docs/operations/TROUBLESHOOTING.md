# Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| `NoValidatedConfig` on startup | Runtime tried to reach `Ready` without a validated config | Call `RuntimeSupervisor::validate_and_stage(&cfg)` first |
| `IllegalTransition` | Attempted a lifecycle move not in the transition table | Consult `docs/architecture/SYSTEM_OVERVIEW.md` |
| `IntegrityBroken` from `verify_integrity` | Underlying store was tampered with, or restored from an inconsistent snapshot | Restore from a known-good backup |
| Every observation rejected as `latency_exceeded` | Simulation / replay clock outside policy | Increase `max_accepted_latency_seconds` for offline replay, or supply a clock closer to observation timestamps |
| Relay envelope rejected `ProhibitedContent` | Content contains a token from `contracts::prohibited::PROHIBITED_TOKENS` | The payload violates the scope boundary — fix the producer |
| Relay envelope rejected `InvalidSignature` | Signing key mismatch between producer and gateway policy | Rotate keys through the sponsor's KMS |
| Relay envelope rejected `ReplayDetected` | Nonce reuse | Ensure nonce generator is fresh per envelope |
