//! Append-only event store on SQLite with a chained integrity digest.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Mutex;
use time::OffsetDateTime;

use crate::migrations;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("integrity chain broken at sequence {sequence} (expected {expected}, got {found})")]
    IntegrityBroken { sequence: u64, expected: String, found: String },
    #[error("event_id already used")]
    DuplicateEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub sequence: u64,
    pub event_type: String,
    pub event_id_hex: String,
    pub actor: String,
    pub runtime_id: String,
    pub target: String,
    pub before_ref: Option<String>,
    pub after_ref: Option<String>,
    pub occurred_at: OffsetDateTime,
    pub payload_json: String,
    pub previous_digest_hex: String,
    pub integrity_digest_hex: String,
    pub correlation_id: Option<String>,
    pub security_context: String,
}

pub struct EventStore {
    inner: Mutex<Connection>,
}

impl EventStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let this = Self { inner: Mutex::new(conn) };
        this.run_migrations()?;
        Ok(this)
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        let this = Self { inner: Mutex::new(conn) };
        this.run_migrations()?;
        Ok(this)
    }

    fn run_migrations(&self) -> Result<(), StoreError> {
        let mut conn = self.inner.lock().unwrap();
        let tx = conn.transaction()?;
        for m in migrations::ALL {
            tx.execute_batch(m.sql)?;
            tx.execute(
                "INSERT OR IGNORE INTO schema_migrations(id,name,applied_at_unix_ns) VALUES(?,?,?)",
                params![m.id, m.name, OffsetDateTime::now_utc().unix_timestamp_nanos() as i64],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Compute the chained integrity digest for a candidate event.
    fn compute_digest(
        prev: &str,
        event_type: &str,
        event_id_hex: &str,
        actor: &str,
        runtime_id: &str,
        target: &str,
        occurred_at_ns: i128,
        payload_json: &str,
        security_context: &str,
    ) -> String {
        let mut h = Sha256::new();
        h.update(prev.as_bytes());
        h.update(event_type.as_bytes());
        h.update(event_id_hex.as_bytes());
        h.update(actor.as_bytes());
        h.update(runtime_id.as_bytes());
        h.update(target.as_bytes());
        h.update(occurred_at_ns.to_be_bytes());
        h.update(payload_json.as_bytes());
        h.update(security_context.as_bytes());
        hex::encode(h.finalize())
    }

    /// Append a new event; the sequence number and previous digest are
    /// resolved from the current tail of the log.
    #[allow(clippy::too_many_arguments)]
    pub fn append(
        &self,
        event_type: &str,
        event_id_hex: &str,
        actor: &str,
        runtime_id: &str,
        target: &str,
        before_ref: Option<&str>,
        after_ref: Option<&str>,
        occurred_at: OffsetDateTime,
        payload: &serde_json::Value,
        correlation_id: Option<&str>,
        security_context: &str,
    ) -> Result<StoredEvent, StoreError> {
        let payload_json = serde_json::to_string(payload)?;
        let mut conn = self.inner.lock().unwrap();
        let tx = conn.transaction()?;

        // Duplicate-event guard.
        let exists: u64 = tx.query_row(
            "SELECT COUNT(*) FROM events WHERE event_id_hex = ?",
            [event_id_hex],
            |r| r.get(0),
        )?;
        if exists > 0 {
            return Err(StoreError::DuplicateEvent);
        }

        // Compute next sequence and previous digest.
        let (next_seq, prev_digest): (u64, String) = tx.query_row(
            "SELECT COALESCE(MAX(sequence),0), COALESCE(
                (SELECT integrity_digest_hex FROM events ORDER BY sequence DESC LIMIT 1),
                'GENESIS'
             ) FROM events",
            [],
            |r| Ok((r.get::<_, i64>(0)? as u64 + 1, r.get::<_, String>(1)?)),
        )?;

        let digest = Self::compute_digest(
            &prev_digest, event_type, event_id_hex, actor, runtime_id, target,
            occurred_at.unix_timestamp_nanos(), &payload_json, security_context,
        );

        tx.execute(
            "INSERT INTO events(
                sequence,event_type,event_id_hex,actor,runtime_id,target,
                before_ref,after_ref,occurred_at_unix_ns,payload_json,
                previous_digest_hex,integrity_digest_hex,correlation_id,security_context
             ) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
            params![
                next_seq as i64, event_type, event_id_hex, actor, runtime_id, target,
                before_ref, after_ref,
                occurred_at.unix_timestamp_nanos() as i64,
                payload_json, prev_digest, digest, correlation_id, security_context,
            ],
        )?;
        tx.commit()?;

        Ok(StoredEvent {
            sequence: next_seq,
            event_type: event_type.into(),
            event_id_hex: event_id_hex.into(),
            actor: actor.into(),
            runtime_id: runtime_id.into(),
            target: target.into(),
            before_ref: before_ref.map(str::to_string),
            after_ref: after_ref.map(str::to_string),
            occurred_at,
            payload_json,
            previous_digest_hex: prev_digest,
            integrity_digest_hex: digest,
            correlation_id: correlation_id.map(str::to_string),
            security_context: security_context.into(),
        })
    }

    /// Walk the chain from GENESIS to the tail and verify every hash link.
    pub fn verify_integrity(&self) -> Result<(), StoreError> {
        let conn = self.inner.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT sequence,event_type,event_id_hex,actor,runtime_id,target,
                    occurred_at_unix_ns,payload_json,previous_digest_hex,
                    integrity_digest_hex,security_context
             FROM events ORDER BY sequence ASC",
        )?;
        let mut expected_prev = "GENESIS".to_string();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let sequence: u64 = row.get::<_, i64>(0)? as u64;
            let event_type: String = row.get(1)?;
            let event_id_hex: String = row.get(2)?;
            let actor: String = row.get(3)?;
            let runtime_id: String = row.get(4)?;
            let target: String = row.get(5)?;
            let occurred_at_ns: i64 = row.get(6)?;
            let payload_json: String = row.get(7)?;
            let previous_digest_hex: String = row.get(8)?;
            let integrity_digest_hex: String = row.get(9)?;
            let security_context: String = row.get(10)?;
            if previous_digest_hex != expected_prev {
                return Err(StoreError::IntegrityBroken {
                    sequence,
                    expected: expected_prev,
                    found: previous_digest_hex,
                });
            }
            let recomputed = Self::compute_digest(
                &previous_digest_hex, &event_type, &event_id_hex, &actor,
                &runtime_id, &target, occurred_at_ns as i128, &payload_json,
                &security_context,
            );
            if recomputed != integrity_digest_hex {
                return Err(StoreError::IntegrityBroken {
                    sequence,
                    expected: recomputed,
                    found: integrity_digest_hex,
                });
            }
            expected_prev = integrity_digest_hex;
        }
        Ok(())
    }

    /// Record a rejected raw observation. Never silently dropped.
    pub fn record_rejected_observation(
        &self,
        observation_id_hex: &str,
        reason: &str,
        raw_json: &str,
    ) -> Result<(), StoreError> {
        let conn = self.inner.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO rejected_observations(
                observation_id_hex,rejected_at_unix_ns,reason,raw_json
             ) VALUES(?,?,?,?)",
            params![
                observation_id_hex,
                OffsetDateTime::now_utc().unix_timestamp_nanos() as i64,
                reason, raw_json,
            ],
        )?;
        Ok(())
    }

    pub fn tail_sequence(&self) -> Result<u64, StoreError> {
        let conn = self.inner.lock().unwrap();
        let s: i64 = conn.query_row(
            "SELECT COALESCE(MAX(sequence),0) FROM events", [], |r| r.get(0)
        )?;
        Ok(s as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use time::OffsetDateTime;

    fn ts(offset_ns: i64) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp_nanos(1_700_000_000_000_000_000_i128 + offset_ns as i128).unwrap()
    }

    #[test]
    fn append_and_verify_chain() {
        let store = EventStore::open_in_memory().unwrap();
        for i in 0..5 {
            store.append(
                "TrackCreated",
                &format!("evt-{i:016x}"),
                "actor", "runtime-1",
                &format!("trk-{i}"),
                None, Some(&format!("state-{i}")),
                ts(i),
                &json!({"i": i}),
                None, "sc0",
            ).unwrap();
        }
        store.verify_integrity().unwrap();
        assert_eq!(store.tail_sequence().unwrap(), 5);
    }

    #[test]
    fn duplicate_event_rejected() {
        let store = EventStore::open_in_memory().unwrap();
        store.append("X","evt-1","a","r","t",None,None, ts(0),
            &json!({}), None, "sc").unwrap();
        let err = store.append("X","evt-1","a","r","t",None,None, ts(1),
            &json!({}), None, "sc").unwrap_err();
        assert!(matches!(err, StoreError::DuplicateEvent));
    }

    #[test]
    fn rejected_observations_are_preserved() {
        let store = EventStore::open_in_memory().unwrap();
        store.record_rejected_observation("obs-1", "invalid_frame", "{}").unwrap();
        // idempotent replace
        store.record_rejected_observation("obs-1", "invalid_frame_v2", "{}").unwrap();
    }

    #[test]
    fn tampering_breaks_integrity() {
        let store = EventStore::open_in_memory().unwrap();
        for i in 0..3 {
            store.append("X", &format!("evt-{i}"), "a", "r", "t", None, None,
                ts(i), &json!({"i": i}), None, "sc").unwrap();
        }
        // Tamper with the payload of event #2
        {
            let conn = store.inner.lock().unwrap();
            conn.execute("UPDATE events SET payload_json='{\"i\":999}' WHERE sequence=2", []).unwrap();
        }
        let err = store.verify_integrity().unwrap_err();
        assert!(matches!(err, StoreError::IntegrityBroken { sequence: 2, .. }));
    }
}
