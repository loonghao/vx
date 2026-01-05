use rstest::rstest;
use std::path::PathBuf;
use tempfile::TempDir;
use vx_cache::{atomic_write_string, read_json_file, write_json_file};

#[rstest]
fn test_atomic_write_string_overwrites_existing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.txt");

    atomic_write_string(&path, "one").unwrap();
    atomic_write_string(&path, "two").unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, "two");
}

#[rstest]
fn test_write_json_and_read_json_roundtrip() {
    let dir = TempDir::new().unwrap();
    let path: PathBuf = dir.path().join("test.json");

    #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    struct Obj {
        a: String,
        b: u32,
    }

    let v = Obj {
        a: "hello".to_string(),
        b: 42,
    };

    write_json_file(&path, &v).unwrap();
    let back: Obj = read_json_file(&path).unwrap();

    assert_eq!(back, v);
}
