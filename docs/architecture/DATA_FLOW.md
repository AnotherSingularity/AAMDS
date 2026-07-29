# Data Flow

```
adapter → RawObservation
       (schema_version = observation_schema())
       ↓
normalization → NormalizedObservation
       (canonical time, canonical position, transformation_chain preserved,
        original value never overwritten)
       ↓
track-management → TrackUpdate → Track
       (deterministic_sequence monotonic per track,
        provenance_root carries every contributing observation)
       ↓
persistence  (append-only)
       ↓                                 ↘
operator-api                              secure-relay
   (read-only)                              (allowlist + signature + policy)
       ↓                                       ↓
operator UI                              authorized peer
```

Invariants:

- `deterministic_sequence` is monotonic per track and stable across replay.
- Every step records a `TransformationStep` in the transformation chain.
- Any subsystem may refuse an input; refused inputs are audited, never silently
  dropped.
- `secure-relay` never sees a `RelayMessageKind` other than the four
  informational variants.
