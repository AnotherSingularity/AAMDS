//! Signing and anti-replay primitives.
//!
//! Signature: baseline uses a keyed HMAC-style SHA-256 over the canonical
//! envelope digest. Sponsor deployments should replace this with a
//! FIPS-validated primitive from a KMS/HSM — the abstraction is a single
//! `sign` / `verify` pair.
//!
//! Anti-replay: nonces are stored in a TTL cache; a repeat within the
//! window is rejected as a replay.

use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use time::OffsetDateTime;

pub fn sign(private_key_material: &[u8], canonical_digest_hex: &str) -> String {
    let mut h = Sha256::new();
    h.update(private_key_material);
    h.update(canonical_digest_hex.as_bytes());
    hex::encode(h.finalize())
}

pub fn verify(private_key_material: &[u8], canonical_digest_hex: &str, signature_hex: &str) -> bool {
    let expected = sign(private_key_material, canonical_digest_hex);
    // Constant-time comparison at the byte level.
    if expected.len() != signature_hex.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (a, b) in expected.as_bytes().iter().zip(signature_hex.as_bytes().iter()) {
        diff |= a ^ b;
    }
    diff == 0
}

/// Canonical digest over the fields that must be integrity-bound by the
/// signature. The scheme is chosen so a change to any of these fields
/// invalidates the signature.
pub fn canonical_envelope_digest(
    message_id: &str,
    destination: &str,
    kind: &str,
    payload_digest_hex: &str,
    created_at: OffsetDateTime,
    expires_at: OffsetDateTime,
    sender: &str,
    anti_replay_nonce_hex: &str,
) -> String {
    let mut h = Sha256::new();
    h.update(message_id.as_bytes());
    h.update(destination.as_bytes());
    h.update(kind.as_bytes());
    h.update(payload_digest_hex.as_bytes());
    h.update(created_at.unix_timestamp_nanos().to_be_bytes());
    h.update(expires_at.unix_timestamp_nanos().to_be_bytes());
    h.update(sender.as_bytes());
    h.update(anti_replay_nonce_hex.as_bytes());
    hex::encode(h.finalize())
}

#[derive(Debug)]
pub struct AntiReplayCache {
    ttl_seconds: i64,
    seen: VecDeque<(String, OffsetDateTime)>,
}

impl AntiReplayCache {
    pub fn new(ttl_seconds: u32) -> Self {
        Self { ttl_seconds: ttl_seconds as i64, seen: VecDeque::new() }
    }

    fn evict(&mut self, now: OffsetDateTime) {
        while let Some((_, t)) = self.seen.front() {
            if (now - *t).whole_seconds() > self.ttl_seconds {
                self.seen.pop_front();
            } else {
                break;
            }
        }
    }

    /// Returns true if `nonce` was accepted (new); false if it is a
    /// replay within the retention window.
    pub fn observe(&mut self, nonce: &str, now: OffsetDateTime) -> bool {
        self.evict(now);
        if self.seen.iter().any(|(n, _)| n == nonce) {
            return false;
        }
        self.seen.push_back((nonce.to_string(), now));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_then_verify_round_trip() {
        let key = b"secret";
        let sig = sign(key, "abc");
        assert!(verify(key, "abc", &sig));
        assert!(!verify(key, "abd", &sig));
        assert!(!verify(b"other", "abc", &sig));
    }

    #[test]
    fn anti_replay_cache_rejects_repeat_within_window() {
        let now = OffsetDateTime::UNIX_EPOCH;
        let mut c = AntiReplayCache::new(60);
        assert!(c.observe("nonce-1", now));
        assert!(!c.observe("nonce-1", now));
    }

    #[test]
    fn anti_replay_cache_forgets_after_ttl() {
        let mut c = AntiReplayCache::new(10);
        let t0 = OffsetDateTime::UNIX_EPOCH;
        c.observe("n", t0);
        let t1 = t0 + time::Duration::seconds(11);
        assert!(c.observe("n", t1));
    }
}
