#[derive(Debug, Clone)]
pub(crate) struct FunctionalRequirement {
    pub(crate) id: String,
    pub(crate) description: String,
}

/// Parse `FR-NNN: description` lines from spec content.
pub(crate) fn parse_functional_requirements(spec: &str) -> Vec<FunctionalRequirement> {
    let mut frs = Vec::new();
    for line in spec.lines() {
        if let Some(pos) = line.find("FR-") {
            let rest = &line[pos..];
            let id_end = rest[3..]
                .find(|c: char| !c.is_ascii_digit())
                .map(|p| p + 3)
                .unwrap_or(rest.len());
            let id = &rest[..id_end];
            let description = if let Some(colon) = rest.find(':') {
                rest[colon + 1..]
                    .trim()
                    .trim_matches('*')
                    .trim()
                    .to_string()
            } else {
                rest.to_string()
            };
            if !description.is_empty() && id.len() > 3 {
                frs.push(FunctionalRequirement {
                    id: id.to_string(),
                    description,
                });
            }
        }
    }
    frs.dedup_by_key(|fr| fr.id.clone());
    frs
}

/// Group FRs into logical WP batches (3-7 FRs per WP).
pub(crate) fn group_frs_into_wps(
    frs: &[FunctionalRequirement],
    max_wps: usize,
) -> Vec<Vec<FunctionalRequirement>> {
    if frs.is_empty() {
        return Vec::new();
    }
    let target_per_wp =
        ((frs.len() as f64) / (max_wps as f64).min(frs.len() as f64)).ceil() as usize;
    let per_wp = target_per_wp.clamp(3, 7);
    frs.chunks(per_wp).map(|chunk| chunk.to_vec()).collect()
}

/// Derive a human-readable WP title from a group of FRs.
pub(crate) fn derive_wp_title(frs: &[FunctionalRequirement], index: usize) -> String {
    if frs.is_empty() {
        return format!("Work Package {index:02}");
    }
    let hint = &frs[0].description;
    let truncated: String = hint.chars().take(50).collect();
    format!("{truncated} (WP{index:02})")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_functional_requirements ───────────────────────────────────

    #[test]
    fn parse_fr_single_line() {
        let spec = "- **FR-001**: User can log in\n";
        let frs = parse_functional_requirements(spec);
        assert_eq!(frs.len(), 1);
        assert_eq!(frs[0].id, "FR-001");
        assert_eq!(frs[0].description, "User can log in");
    }

    #[test]
    fn parse_fr_multiple_lines() {
        let spec = "- **FR-001**: Login\n- **FR-002**: Logout\n- **FR-003**: Password reset\n";
        let frs = parse_functional_requirements(spec);
        assert_eq!(frs.len(), 3);
        assert_eq!(frs[0].id, "FR-001");
        assert_eq!(frs[1].id, "FR-002");
        assert_eq!(frs[2].id, "FR-003");
    }

    #[test]
    fn parse_fr_without_colon() {
        let spec = "FR-010 is referenced here\n";
        let frs = parse_functional_requirements(spec);
        assert_eq!(frs.len(), 1);
        assert_eq!(frs[0].id, "FR-010");
    }

    #[test]
    fn parse_fr_strips_bold_markers() {
        let spec = "- **FR-001**: **Important** feature\n";
        let frs = parse_functional_requirements(spec);
        assert_eq!(frs.len(), 1);
        assert!(!frs[0].description.contains("**"));
    }

    #[test]
    fn parse_fr_deduplicates_by_id() {
        let spec = "- **FR-001**: First\n- **FR-001**: Duplicate\n";
        let frs = parse_functional_requirements(spec);
        assert_eq!(frs.len(), 1);
        assert_eq!(frs[0].id, "FR-001");
    }

    #[test]
    fn parse_fr_skips_empty_description() {
        let spec = "- **FR-001**:   \n";
        let frs = parse_functional_requirements(spec);
        assert!(frs.is_empty(), "empty description should be skipped");
    }

    #[test]
    fn parse_fr_skips_non_fr_lines() {
        let spec = "## Requirements\nSome text\nMore text\n";
        let frs = parse_functional_requirements(spec);
        assert!(frs.is_empty());
    }

    #[test]
    fn parse_fr_empty_spec() {
        let frs = parse_functional_requirements("");
        assert!(frs.is_empty());
    }

    #[test]
    fn parse_fr_inline_fr_not_at_line_start() {
        let spec = "See FR-001 for details\n";
        let frs = parse_functional_requirements(spec);
        assert_eq!(frs.len(), 1);
        assert_eq!(frs[0].id, "FR-001");
    }

    #[test]
    fn parse_fr_large_id() {
        let spec = "- **FR-9999999**: Large ID\n";
        let frs = parse_functional_requirements(spec);
        assert_eq!(frs.len(), 1);
        assert_eq!(frs[0].id, "FR-9999999");
    }

    // ── group_frs_into_wps ──────────────────────────────────────────────

    fn make_frs(count: usize) -> Vec<FunctionalRequirement> {
        (1..=count)
            .map(|i| FunctionalRequirement {
                id: format!("FR-{i:03}"),
                description: format!("Requirement {i}"),
            })
            .collect()
    }

    #[test]
    fn group_frs_empty() {
        let groups = group_frs_into_wps(&[], 5);
        assert!(groups.is_empty());
    }

    #[test]
    fn group_frs_fewer_than_max_wps() {
        let frs = make_frs(3);
        let groups = group_frs_into_wps(&frs, 5);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
    }

    #[test]
    fn group_frs_even_split() {
        let frs = make_frs(6);
        let groups = group_frs_into_wps(&frs, 3);
        // per_wp = clamp(ceil(6/3), 3, 7) = clamp(2, 3, 7) = 3
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].len(), 3);
        assert_eq!(groups[1].len(), 3);
    }

    #[test]
    fn group_frs_many_wps() {
        let frs = make_frs(20);
        let groups = group_frs_into_wps(&frs, 2);
        // per_wp = clamp(ceil(20/2), 3, 7) = clamp(10, 3, 7) = 7
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].len(), 7);
        assert_eq!(groups[1].len(), 7);
        assert_eq!(groups[2].len(), 6);
    }

    #[test]
    fn group_frs_preserves_order() {
        let frs = make_frs(9);
        let groups = group_frs_into_wps(&frs, 2);
        let all_ids: Vec<String> = groups.into_iter().flatten().map(|fr| fr.id).collect();
        let expected: Vec<String> = (1..=9).map(|i| format!("FR-{i:03}")).collect();
        assert_eq!(all_ids, expected);
    }

    // ── derive_wp_title ─────────────────────────────────────────────────

    #[test]
    fn derive_title_empty_frs() {
        let title = derive_wp_title(&[], 1);
        assert_eq!(title, "Work Package 01");
    }

    #[test]
    fn derive_title_uses_first_fr() {
        let frs = make_frs(3);
        let title = derive_wp_title(&frs, 2);
        assert!(title.contains("Requirement 1"));
        assert!(title.contains("WP02"));
    }

    #[test]
    fn derive_title_truncates_long_description() {
        let frs = vec![FunctionalRequirement {
            id: "FR-001".to_string(),
            description: "A".repeat(100),
        }];
        let title = derive_wp_title(&frs, 1);
        assert!(title.len() < 100);
        assert!(title.contains("WP01"));
    }

    #[test]
    fn derive_title_zero_padded_index() {
        let title = derive_wp_title(&[], 42);
        assert!(title.contains("42"));
    }
}
