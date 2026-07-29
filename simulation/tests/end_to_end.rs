//! End-to-end pipeline test: adapter -> normalization -> tracking.
//! Referenced by `tools/verify.sh e2e`.

use aeon_simulation::pipeline::run_pipeline;
use aeon_simulation::scenarios::single_clean_track;
use time::OffsetDateTime;

#[test]
fn end_to_end_single_clean_track_produces_a_stable_active_track() {
    let mut s = single_clean_track();
    let far = OffsetDateTime::UNIX_EPOCH + time::Duration::days(365 * 10);
    let out = run_pipeline("e2e", &mut s.adapter, s.policy, || far).unwrap();
    assert!(out.updates.len() >= 5);
    let last = out.updates.last().unwrap();
    assert!(matches!(last.new_state.status,
        aeon_contracts::track::TrackStatus::Active
      | aeon_contracts::track::TrackStatus::Tentative));
    // Provenance root carries the source-observation identifier
    assert!(!last.new_state.provenance_root.source_observations.is_empty()
         || !last.contributing_observations.is_empty());
    assert_eq!(out.rejected_count, 0);
}
