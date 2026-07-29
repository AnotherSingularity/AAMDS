# Developer profile

- **Runtime**: in-process, `cargo run -p aeon-operator-api` + synthetic
  adapter driven by the simulation crate.
- **Persistence**: SQLite file (`./aeon-developer.sqlite`).
- **Relay**: local simulator; no external network.
- **UI**: `operator-interface/index.html` served by any static host
  (`python3 -m http.server 5173`).

```
deployment/developer/build.sh
target/deploy/developer/
```
