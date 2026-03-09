//! Version comparison and matching utilities
//!
//! Pure utility functions for comparing and matching version strings.
//! These functions have no dependencies on the rest of the executor module.

use std::cmp::Ordering;

/// Compare two version strings (semver-aware).
///
/// Strips leading 'v' prefix, splits on '.', and compares numeric parts.
/// Pre-release suffixes (after '-') are ignored for the numeric comparison.
pub fn compare_versions(a: &str, b: &str) -> Ordering {
    let a_clean = a.trim_start_matches('v');
    let b_clean = b.trim_start_matches('v');

    let a_parts: Vec<u64> = a_clean
        .split('.')
        .filter_map(|s| s.split('-').next())
        .filter_map(|s| s.parse().ok())
        .collect();
    let b_parts: Vec<u64> = b_clean
        .split('.')
        .filter_map(|s| s.split('-').next())
        .filter_map(|s| s.parse().ok())
        .collect();

    for (ap, bp) in a_parts.iter().zip(b_parts.iter()) {
        match ap.cmp(bp) {
            Ordering::Equal => continue,
            other => return other,
        }
    }

    a_parts.len().cmp(&b_parts.len())
}

/// Find a matching version from a list of installed versions.
///
/// First tries exact match, then prefix match for partial versions.
/// Returns the latest matching version.
pub fn find_matching_version(requested: &str, installed: &[String]) -> Option<String> {
    // First try exact match
    if installed.contains(&requested.to_string()) {
        return Some(requested.to_string());
    }

    // Try prefix match for partial versions
    let mut matches: Vec<&String> = installed
        .iter()
        .filter(|v| {
            v.starts_with(requested)
                && (v.len() == requested.len() || v.chars().nth(requested.len()) == Some('.'))
        })
        .collect();

    if matches.is_empty() {
        return None;
    }

    // Sort and return the latest matching version
    matches.sort_by(|a, b| compare_versions(a, b));
    matches.last().map(|s| (*s).clone())
}
