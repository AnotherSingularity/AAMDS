//! Adversarial tests for the event store.
//! Section 23.4 of the implementation directive.

use aeon_persistence::EventStore;
use serde_json::json;
use time::OffsetDateTime;

fn ts(o: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp_nanos(
        1_700_000_000_000_000_000_i128 + o as i128
    ).unwrap()
}

#[test]
fn audit_deletion_attempt_is_detected() {
    let store = EventStore::open_in_memory().unwrap();
    for i in 0..5 {
        store.append("X", &format!("evt-{i}"), "a", "r", "t", None, None,
            ts(i), &json!({"i": i}), None, "sc").unwrap();
    }
    // Try to "hide" event 3 by deleting it. Because the store is
    // append-only we simulate a tamper via direct SQL; the integrity
    // walk must catch it.
    use rusqlite::Connection;
    let path = tempfile::NamedTempFile::new().unwrap();
    let file_store = EventStore::open(path.path()).unwrap();
    for i in 0..5 {
        file_store.append("X", &format!("f-{i}"), "a", "r", "t", None, None,
            ts(i), &json!({"i": i}), None, "sc").unwrap();
    }
    // Reopen a raw connection to the same file and delete event 3.
    let raw = Connection::open(path.path()).unwrap();
    raw.execute("DELETE FROM events WHERE sequence = 3", []).unwrap();
    drop(raw);
    let reopened = EventStore::open(path.path()).unwrap();
    assert!(reopened.verify_integrity().is_err(),
        "deleting event 3 must break the chain");
}

#[test]
fn duplicate_event_id_is_rejected_across_the_store_lifetime() {
    let store = EventStore::open_in_memory().unwrap();
    store.append("X", "dup", "a", "r", "t", None, None, ts(0),
        &json!({}), None, "sc").unwrap();
    assert!(store.append("X", "dup", "a", "r", "t", None, None, ts(1),
        &json!({}), None, "sc").is_err());
}
