#![no_main]

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use libfuzzer_sys::fuzz_target;
use std::str::FromStr;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };

    let _ = serde_json::from_str::<Feature>(input);
    let _ = FeatureState::from_str(input);
    let _ = Feature::slug_from_name(input);
});
