# Operator Interface (thin client)

The baseline operator interface is intentionally minimal: a single
`index.html` file that consumes the versioned operator API
(`operator-api/`) and renders the recognized operational picture.

This is deliberate:

- The interface must not carry business-critical fusion or security
  logic (directive section 14). Fusion happens in `track-management`;
  security enforcement happens in `secure-relay`.
- A single-file client is trivially reproducible from a clean checkout
  with `python3 -m http.server` — no npm, no build step.
- A production deployment typically replaces this with a fully-built
  React application. The `package.json` and `tsconfig.json` here define
  the scaffolding for that transition; adopting it requires a
  sponsor-owned build pipeline.

## Run (baseline)

```
# in one shell:
cargo run -p aeon-operator-api
# in another:
cd operator-interface && python3 -m http.server 5173
# then open http://127.0.0.1:5173/
```

## Views

The single page exposes the directive-required views:

- Operational overview
- Track list + track detail
- Observation provenance (via track detail)
- Confidence + uncertainty
- Source agreement + conflict
- System health / adapter health / relay status
- Alert management (with acknowledge action)
- Replay control indicator
- Version / configuration / model status

The interface distinguishes measured, estimated, stale, conflicting, and
untrusted data using the `Known<T>` variant on the wire; it does not
hide missing data.
