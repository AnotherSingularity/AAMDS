# Backup and Restore

## SQLite (developer / edge / disconnected)

Backup:
```
sqlite3 aeon.sqlite ".backup 'aeon-$(date -u +%Y%m%dT%H%M%SZ).sqlite'"
```
Restore:
```
cp aeon-YYYYMMDDTHHMMSSZ.sqlite aeon.sqlite
# then start the runtime and confirm:
#   EventStore::verify_integrity == Ok
```

## Postgres-compatible (fixed-site / data-centre / private-cloud)

Backup: `pg_dump --format=custom aeon > aeon.dump`
Restore: `pg_restore --clean --create -d postgres aeon.dump`

Restore validity is verified by `verify_integrity` walking the audit
chain to the tail. A restore that leaves the chain incomplete fails
readiness.
