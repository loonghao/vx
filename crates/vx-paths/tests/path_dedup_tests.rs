//! PATH composition regression tests.

use vx_paths::{join_paths_simple, path_separator, prepend_to_path, split_path};

#[test]
fn prepend_to_path_deduplicates_nested_managed_blocks() {
    let separator = path_separator().to_string();
    let managed = join_paths_simple(&["/vx/store/node/bin", "/vx/bin"]);
    let original = join_paths_simple(&[managed.as_str(), "/system/bin"]);

    let result = prepend_to_path(&original, &[managed.as_str()]);
    let parts: Vec<&str> = split_path(&result).collect();

    assert_eq!(
        parts,
        vec!["/vx/store/node/bin", "/vx/bin", "/system/bin"],
        "nested vx invocations must not duplicate managed PATH entries (separator={separator})"
    );
}
