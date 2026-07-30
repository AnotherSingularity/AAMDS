# Network Integration Checklist

## Ingress (sensor adapters)
- [ ] Each configured adapter reaches its authorized source over the
      sponsor's approved transport.
- [ ] Adapter `AdapterCapability` accurately reflects what the source
      provides.
- [ ] Adapter conformance harness passes against a sponsor-provided
      sample stream.

## Egress (relay)
- [ ] Each `DestinationPolicy` entry has a real destination on the
      sponsor's network.
- [ ] Signature verification is performed by the peer.
- [ ] Anti-replay window is aligned with the peer's own retention
      window.
- [ ] Rate limits align with the peer's ingest capacity.

## Identity provider
- [ ] Operator API is fronted by the sponsor's authenticating proxy or
      role-guard middleware — the API itself is unauthenticated (see
      `docs/security/ACCESS_CONTROL_MATRIX.md`).

## Observability
- [ ] Structured logs from `tracing_subscriber::fmt().json()` reach the
      sponsor's SIEM.
- [ ] Prometheus / OTLP scrapes reach the sponsor's monitoring stack
      (if enabled by the profile).

## Sign-off
- Network engineer:
- Date:
- Result (PASS / FAIL):
