//! Link strategy tests

use tempfile::TempDir;
use vx_paths::{LinkStrategy, link};

#[test]
fn test_link_strategy_auto() {
    let strategy = LinkStrategy::auto();

    // Should return a valid strategy for any platform
    assert!(matches!(
        strategy,
        LinkStrategy::HardLink
            | LinkStrategy::SymLink
            | LinkStrategy::CopyOnWrite
            | LinkStrategy::Copy
    ));
}

#[test]
fn test_link_strategy_name() {
    assert_eq!(LinkStrategy::HardLink.name(), "hard link");
    assert_eq!(LinkStrategy::SymLink.name(), "symbolic link");
    assert_eq!(LinkStrategy::CopyOnWrite.name(), "copy-on-write");
    assert_eq!(LinkStrategy::Copy.name(), "copy");
}

#[test]
fn test_create_link_copy() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("src");
    let dst = temp_dir.path().join("dst");

    // Create source file
    std::fs::write(&src, "test content").unwrap();

    // Copy
    link::create_link(&src, &dst, LinkStrategy::Copy).unwrap();

    // Verify
    assert!(dst.exists());
    assert_eq!(std::fs::read_to_string(&dst).unwrap(), "test content");
}

#[test]
fn test_create_link_copy_directory() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("src_dir");
    let dst = temp_dir.path().join("dst_dir");

    // Create source directory with files
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("file1.txt"), "content1").unwrap();
    std::fs::write(src.join("file2.txt"), "content2").unwrap();

    // Copy
    link::create_link(&src, &dst, LinkStrategy::Copy).unwrap();

    // Verify
    assert!(dst.exists());
    assert!(dst.join("file1.txt").exists());
    assert!(dst.join("file2.txt").exists());
    assert_eq!(
        std::fs::read_to_string(dst.join("file1.txt")).unwrap(),
        "content1"
    );
}

#[test]
fn test_link_directory() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("src_dir");
    let dst = temp_dir.path().join("dst_dir");

    // Create source directory with files
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("file1.txt"), "content1").unwrap();
    std::fs::create_dir_all(src.join("subdir")).unwrap();
    std::fs::write(src.join("subdir/file2.txt"), "content2").unwrap();

    // Link
    let result = link::link_directory(&src, &dst).unwrap();

    // Verify
    assert!(result.success);
    assert!(dst.exists());
    assert!(dst.join("file1.txt").exists());
    assert!(dst.join("subdir/file2.txt").exists());
}
