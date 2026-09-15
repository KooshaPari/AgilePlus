//! Integration tests for the FeatureState lifecycle state machine.
//!
//! These tests exercise the full state machine including transitions,
//! error handling, serialization, and edge cases.

use std::str::FromStr;

use traceability_core::FeatureState;

// ---------------------------------------------------------------------------
// Valid transition chain
// ---------------------------------------------------------------------------

/// Walk every forward transition and verify the chain produces correct
/// TransitionResult values with timestamps.
#[test]
fn full_forward_chain_produces_timestamps() {
    let all = FeatureState::all();
    let mut prev = all[0];
    for &next in &all[1..] {
        let result = prev.transition(next).unwrap();
        assert_eq!(result.transition.from, prev);
        assert_eq!(result.transition.to, next);
        assert!(!result.timestamp.to_string().is_empty());
        prev = next;
    }
}

/// Verify ordinals are strictly increasing across all 8 states.
#[test]
fn ordinals_cover_zero_to_seven() {
    let all = FeatureState::all();
    assert_eq!(all.len(), 8);
    for (i, state) in all.iter().enumerate() {
        assert_eq!(state.ordinal(), i as u8);
    }
}

// ---------------------------------------------------------------------------
// Invalid transitions
// ---------------------------------------------------------------------------

/// Every state rejects same-state transition.
#[test]
fn same_state_transition_always_rejected() {
    for &state in FeatureState::all() {
        let err = state.transition(state);
        assert!(err.is_err());
        let err = err.unwrap_err();
        assert_eq!(err.from, state.to_string());
        assert_eq!(err.to, state.to_string());
        assert!(err.reason.contains("not an allowed"));
    }
}

/// Every state rejects transitions to a strictly earlier ordinal.
#[test]
fn backward_transitions_always_rejected() {
    let all = FeatureState::all();
    // For each state, try transitioning to every state with a lower ordinal.
    for (i, &from) in all.iter().enumerate() {
        for &target in &all[..i] {
            let result = from.transition(target);
            assert!(
                result.is_err(),
                "backward transition {from:?} -> {target:?} should be rejected"
            );
        }
    }
}

/// Skip-forward transitions (e.g. Created -> Planned) are rejected.
#[test]
fn skip_forward_transitions_rejected() {
    let all = FeatureState::all();
    for window in all.windows(3) {
        let result = window[0].transition(window[2]);
        assert!(
            result.is_err(),
            "skip {:?} -> {:?} should be rejected",
            window[0],
            window[2]
        );
    }
}

// ---------------------------------------------------------------------------
// FromStr / Display roundtrips
// ---------------------------------------------------------------------------

/// All 8 state strings parse correctly via FromStr.
#[test]
fn from_str_all_valid_variants() {
    let cases = [
        ("created", FeatureState::Created),
        ("specified", FeatureState::Specified),
        ("researched", FeatureState::Researched),
        ("planned", FeatureState::Planned),
        ("implementing", FeatureState::Implementing),
        ("validated", FeatureState::Validated),
        ("shipped", FeatureState::Shipped),
        ("retrospected", FeatureState::Retrospected),
    ];
    for (s, expected) in cases {
        let parsed = FeatureState::from_str(s).unwrap();
        assert_eq!(parsed, expected);
        assert_eq!(parsed.to_string(), s);
    }
}

/// FromStr rejects garbage input.
#[test]
fn from_str_rejects_various_garbage() {
    for garbage in ["", "CREATED", "Created", "shipped!", "x", "retro"] {
        assert!(
            FeatureState::from_str(garbage).is_err(),
            "FromStr should reject: {garbage:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Serde roundtrips
// ---------------------------------------------------------------------------

/// JSON serialization uses lowercase strings.
#[test]
fn serde_uses_lowercase() {
    for state in FeatureState::all() {
        let json = serde_json::to_string(state).unwrap();
        assert_eq!(json, format!("\"{}\"", state));
    }
}

/// Full roundtrip for every variant.
#[test]
fn serde_roundtrip_all_variants() {
    for &state in FeatureState::all() {
        let json = serde_json::to_string(&state).unwrap();
        let back: FeatureState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, state);
    }
}

// ---------------------------------------------------------------------------
// LifecycleError display
// ---------------------------------------------------------------------------

#[test]
fn lifecycle_error_display_format() {
    let result = FeatureState::Shipped.transition(FeatureState::Created);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("shipped"));
    assert!(msg.contains("created"));
    assert!(msg.contains("not an allowed"));
}

// ---------------------------------------------------------------------------
// TransitionResult timestamp is recent
// ---------------------------------------------------------------------------

#[test]
fn transition_result_timestamp_is_recent() {
    let before = chrono::Utc::now();
    let result = FeatureState::Created
        .transition(FeatureState::Specified)
        .unwrap();
    let after = chrono::Utc::now();
    assert!(result.timestamp >= before);
    assert!(result.timestamp <= after);
}
