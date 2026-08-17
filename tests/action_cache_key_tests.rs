//! Regression tests for the setup Action cache key contract.

use std::{fs, path::PathBuf};

use rstest::{fixture, rstest};

#[fixture]
fn action_yaml() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("action.yml");
    fs::read_to_string(path).expect("action.yml should be readable")
}

#[rstest]
fn cache_keys_are_arch_scoped_and_keep_the_custom_prefix(action_yaml: String) {
    let cache_step = action_yaml
        .split_once("    - name: Cache vx tools")
        .expect("cache step should exist")
        .1
        .split_once("\n    # Install vx")
        .expect("cache step should end before installation")
        .0;
    let key_templates: Vec<_> = cache_step
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            line.strip_prefix("key: ").or_else(|| {
                line.starts_with("${{ inputs.cache-key-prefix }}-")
                    .then_some(line)
            })
        })
        .collect();

    assert_eq!(
        key_templates,
        [
            "${{ inputs.cache-key-prefix }}-${{ runner.os }}-${{ runner.arch }}-${{ inputs.version }}-${{ inputs.tools }}-${{ hashFiles('vx.toml', '.vx.toml', 'vx.lock') }}",
            "${{ inputs.cache-key-prefix }}-${{ runner.os }}-${{ runner.arch }}-${{ inputs.version }}-",
            "${{ inputs.cache-key-prefix }}-${{ runner.os }}-${{ runner.arch }}-",
        ]
    );

    let prefix_input = action_yaml
        .split_once("  cache-key-prefix:")
        .expect("cache-key-prefix input should exist")
        .1
        .split_once("  setup:")
        .expect("cache-key-prefix input should precede setup")
        .0;
    assert!(prefix_input.contains("    default: \"vx-tools\""));
}
