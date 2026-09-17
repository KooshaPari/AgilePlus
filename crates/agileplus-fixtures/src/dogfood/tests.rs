// SPDX-License-Identifier: MIT OR Apache-2.0
//! Contract tests for the dogfood seed data.

use super::*;

#[test]
fn seed_creates_all_features() {
    let (features, _work_packages) = seed_dogfood_features();
    assert_eq!(
        features.len(),
        37,
        "Should create 4 AgilePlus + 33 SpecKitty features"
    );
}

#[test]
fn seed_creates_work_packages_for_all_features() {
    let (features, work_packages) = seed_dogfood_features();
    assert_eq!(
        work_packages.len(),
        features.len(),
        "Should have work packages for all features"
    );
    for f in &features {
        assert!(
            work_packages.contains_key(&f.id),
            "Missing WPs for feature {}",
            f.id
        );
    }
}

#[test]
fn seed_speckitty_features_tagged() {
    let (features, _work_packages) = seed_dogfood_features();
    for f in &features[4..] {
        assert!(
            f.labels.contains(&"specKitty".to_owned()),
            "SpecKitty feature {} missing specKitty label",
            f.slug
        );
    }
}

#[test]
fn seed_feature_ids_are_sequential_one_through_thirty_seven() {
    let (features, _) = seed_dogfood_features();
    let ids: Vec<i64> = features.iter().map(|f| f.id).collect();
    assert_eq!(ids, (1..=37).collect::<Vec<_>>());
}

#[test]
fn seed_feature_slugs_are_unique() {
    let (features, _) = seed_dogfood_features();
    let mut slugs: Vec<&str> = features.iter().map(|f| f.slug.as_str()).collect();
    slugs.sort_unstable();
    let before = slugs.len();
    slugs.dedup();
    assert_eq!(slugs.len(), before, "duplicate feature slugs in seed data");
}

#[test]
fn seed_feature_names_are_non_empty() {
    let (features, _) = seed_dogfood_features();
    for f in &features {
        assert!(!f.friendly_name.is_empty(), "empty name for {}", f.slug);
        assert!(!f.slug.is_empty());
    }
}

#[test]
fn seed_agileplus_features_one_to_three_are_shipped() {
    let (features, _) = seed_dogfood_features();
    for f in features.iter().take(3) {
        assert_eq!(f.state, FeatureState::Shipped, "{} not shipped", f.slug);
    }
}

#[test]
fn seed_feature_four_is_implementing_with_labels() {
    let (features, _) = seed_dogfood_features();
    let f4 = features.iter().find(|f| f.id == 4).expect("feature 4");
    assert_eq!(f4.slug, "004-modules-and-cycles");
    assert_eq!(f4.state, FeatureState::Implementing);
    assert_eq!(f4.labels, vec!["organization", "planning"]);
}

#[test]
fn seed_speckitty_features_start_at_id_five_and_are_shipped() {
    let (features, _) = seed_dogfood_features();
    let speckitty: Vec<_> = features.iter().filter(|f| f.id >= 5).collect();
    assert_eq!(speckitty.len(), 33, "expected 33 SpecKitty reference specs");
    for f in speckitty {
        assert_eq!(f.state, FeatureState::Shipped);
        assert!(f.slug.starts_with("sk-"), "bad slug {}", f.slug);
    }
}

#[test]
fn seed_all_features_are_project_scoped_and_labelled() {
    let (features, _) = seed_dogfood_features();
    for f in &features {
        assert_eq!(f.project_id, Some(1), "{} missing project", f.slug);
        assert!(!f.labels.is_empty(), "{} missing labels", f.slug);
    }
}

#[test]
fn seed_work_package_counts_match_spec() {
    let (_, work_packages) = seed_dogfood_features();
    assert_eq!(work_packages[&1].len(), 4);
    assert_eq!(work_packages[&2].len(), 3);
    assert_eq!(work_packages[&3].len(), 4);
    assert_eq!(work_packages[&4].len(), 3);
    for id in 5..=37 {
        assert_eq!(work_packages[&id].len(), 2, "feature {id} wp count");
    }
}

#[test]
fn seed_work_package_ids_are_unique_and_contiguous_from_one() {
    let (_, work_packages) = seed_dogfood_features();
    let mut ids: Vec<i64> = work_packages.values().flatten().map(|wp| wp.id).collect();
    ids.sort_unstable();
    let total = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), total, "duplicate work package ids");
    assert_eq!(ids, (1..=total as i64).collect::<Vec<_>>());
    assert_eq!(total, 80);
}

#[test]
fn seed_work_packages_are_keyed_by_owning_feature() {
    let (_, work_packages) = seed_dogfood_features();
    for (feature_id, wps) in &work_packages {
        for wp in wps {
            assert_eq!(wp.feature_id, *feature_id, "wp {} misfiled", wp.id);
        }
    }
}

#[test]
fn seed_work_package_sequences_start_at_one_and_increment() {
    let (_, work_packages) = seed_dogfood_features();
    for (feature_id, wps) in &work_packages {
        let sequences: Vec<i32> = wps.iter().map(|wp| wp.sequence).collect();
        let expected: Vec<i32> = (1..=wps.len() as i32).collect();
        assert_eq!(sequences, expected, "feature {feature_id} sequence");
    }
}

#[test]
fn seed_work_package_states_match_expected_progress() {
    let (_, work_packages) = seed_dogfood_features();
    let done_count = work_packages
        .values()
        .flatten()
        .filter(|wp| wp.state == WpState::Done)
        .count();
    assert_eq!(done_count, 78, "only wp13/wp14 are not Done");

    let wp13 = work_packages[&4]
        .iter()
        .find(|wp| wp.id == 13)
        .expect("wp 13");
    assert_eq!(wp13.state, WpState::Doing);
    let wp14 = work_packages[&4]
        .iter()
        .find(|wp| wp.id == 14)
        .expect("wp 14");
    assert_eq!(wp14.state, WpState::Planned);
}

#[test]
fn seed_work_packages_have_non_empty_titles_and_criteria() {
    let (_, work_packages) = seed_dogfood_features();
    for wp in work_packages.values().flatten() {
        assert!(!wp.title.is_empty(), "wp {} has no title", wp.id);
        assert!(
            !wp.acceptance_criteria.is_empty(),
            "wp {} has no acceptance criteria",
            wp.id
        );
    }
}

#[test]
fn seed_agileplus_work_packages_declare_file_scope_where_expected() {
    let (_, work_packages) = seed_dogfood_features();
    let scoped: Vec<i64> = work_packages[&1]
        .iter()
        .map(|wp| wp.id)
        .chain(work_packages[&2].iter().map(|wp| wp.id))
        .chain(work_packages[&3].iter().map(|wp| wp.id))
        .collect();
    for id in scoped {
        let wp = work_packages
            .values()
            .flatten()
            .find(|wp| wp.id == id)
            .expect("wp");
        assert!(!wp.file_scope.is_empty(), "wp {id} missing file scope");
    }
}

#[test]
fn seed_is_deterministic_across_calls() {
    let (a_features, a_wps) = seed_dogfood_features();
    let (b_features, b_wps) = seed_dogfood_features();

    let a_ids: Vec<i64> = a_features.iter().map(|f| f.id).collect();
    let b_ids: Vec<i64> = b_features.iter().map(|f| f.id).collect();
    assert_eq!(a_ids, b_ids);

    let a_slugs: Vec<&str> = a_features.iter().map(|f| f.slug.as_str()).collect();
    let b_slugs: Vec<&str> = b_features.iter().map(|f| f.slug.as_str()).collect();
    assert_eq!(a_slugs, b_slugs);

    for id in a_wps.keys() {
        let a: Vec<i64> = a_wps[id].iter().map(|wp| wp.id).collect();
        let b: Vec<i64> = b_wps[id].iter().map(|wp| wp.id).collect();
        assert_eq!(a, b, "feature {id} wp ids differ between seeds");
    }
}

#[test]
fn seed_helpers_transition_to_requested_state() {
    let mut feature = Feature::new("x", "X", [0u8; 32], Some("main"));
    drive_to_state(&mut feature, FeatureState::Retrospected);
    assert_eq!(feature.state, FeatureState::Retrospected);

    let mut planned = Feature::new("y", "Y", [0u8; 32], Some("main"));
    drive_to_state(&mut planned, FeatureState::Planned);
    assert_eq!(planned.state, FeatureState::Planned);

    let untouched = Feature::new("z", "Z", [0u8; 32], Some("main"));
    assert_eq!(untouched.state, FeatureState::Created);
}

#[test]
fn seed_shipped_wp_helper_assigns_sequential_ids() {
    let wps = make_shipped_wps(9, 100, &["a", "b", "c"]);
    assert_eq!(wps.len(), 3);
    assert_eq!(wps[0].id, 100);
    assert_eq!(wps[1].id, 101);
    assert_eq!(wps[2].id, 102);
    assert_eq!(wps[0].sequence, 1);
    assert_eq!(wps[2].sequence, 3);
    for wp in &wps {
        assert_eq!(wp.feature_id, 9);
        assert_eq!(wp.state, WpState::Done);
    }
}
