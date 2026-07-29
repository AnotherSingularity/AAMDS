//! Embedded SQL migrations.
//!
//! Migrations are stored in-order as `(id, name, sql)`. `EventStore::open`
//! records each applied migration in `schema_migrations` — attempting to
//! open a database with an unknown migration installed fails closed.

pub struct Migration {
    pub id: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const ALL: &[Migration] = &[
    Migration {
        id: 1,
        name: "0001_events_and_audit",
        sql: r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                id       INTEGER PRIMARY KEY,
                name     TEXT NOT NULL UNIQUE,
                applied_at_unix_ns INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS events (
                sequence           INTEGER PRIMARY KEY,   -- monotone, assigned by store
                event_type         TEXT NOT NULL,
                event_id_hex       TEXT NOT NULL UNIQUE,
                actor              TEXT NOT NULL,
                runtime_id         TEXT NOT NULL,
                target             TEXT NOT NULL,
                before_ref         TEXT,
                after_ref          TEXT,
                occurred_at_unix_ns INTEGER NOT NULL,
                payload_json       TEXT NOT NULL,
                previous_digest_hex TEXT NOT NULL,
                integrity_digest_hex TEXT NOT NULL,
                correlation_id     TEXT,
                security_context   TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS rejected_observations (
                observation_id_hex TEXT PRIMARY KEY,
                rejected_at_unix_ns INTEGER NOT NULL,
                reason             TEXT NOT NULL,
                raw_json           TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS configuration_versions (
                config_id_hex     TEXT PRIMARY KEY,
                activated_at_unix_ns INTEGER NOT NULL,
                actor             TEXT NOT NULL,
                config_json       TEXT NOT NULL
            );
        "#,
    },
];
