//! Tests for RuntimeRequest parsing

use rstest::rstest;
use vx_resolver::RuntimeRequest;

#[rstest]
#[case("yarn", "yarn", None)]
#[case("node", "node", None)]
#[case("npm", "npm", None)]
fn test_parse_name_only(
    #[case] input: &str,
    #[case] expected_name: &str,
    #[case] expected_version: Option<&str>,
) {
    let req = RuntimeRequest::parse(input);
    assert_eq!(req.name, expected_name);
    assert_eq!(req.version.as_deref(), expected_version);
}

#[rstest]
#[case("yarn@1.21.1", "yarn", Some("1.21.1"))]
#[case("node@20", "node", Some("20"))]
#[case("node@20.10.0", "node", Some("20.10.0"))]
#[case("go@1.22", "go", Some("1.22"))]
fn test_parse_with_version(
    #[case] input: &str,
    #[case] expected_name: &str,
    #[case] expected_version: Option<&str>,
) {
    let req = RuntimeRequest::parse(input);
    assert_eq!(req.name, expected_name);
    assert_eq!(req.version.as_deref(), expected_version);
}

#[rstest]
#[case("node@^18.0.0", "node", Some("^18.0.0"))]
#[case("node@~18.0.0", "node", Some("~18.0.0"))]
#[case("node@>=18", "node", Some(">=18"))]
fn test_parse_with_semver_constraint(
    #[case] input: &str,
    #[case] expected_name: &str,
    #[case] expected_version: Option<&str>,
) {
    let req = RuntimeRequest::parse(input);
    assert_eq!(req.name, expected_name);
    assert_eq!(req.version.as_deref(), expected_version);
}

#[test]
fn test_parse_empty_version() {
    let req = RuntimeRequest::parse("yarn@");
    assert_eq!(req.name, "yarn");
    assert_eq!(req.version, None);
}

#[test]
fn test_display() {
    let req = RuntimeRequest::with_version("yarn", "1.21.1");
    assert_eq!(format!("{}", req), "yarn@1.21.1");

    let req = RuntimeRequest::new("yarn");
    assert_eq!(format!("{}", req), "yarn");
}

#[test]
fn test_version_or_latest() {
    let req = RuntimeRequest::new("yarn");
    assert_eq!(req.version_or_latest(), "latest");

    let req = RuntimeRequest::with_version("yarn", "1.21.1");
    assert_eq!(req.version_or_latest(), "1.21.1");
}

#[test]
fn test_has_version() {
    let req = RuntimeRequest::new("yarn");
    assert!(!req.has_version());

    let req = RuntimeRequest::with_version("yarn", "1.21.1");
    assert!(req.has_version());
}

#[test]
fn test_from_str() {
    let req: RuntimeRequest = "yarn@1.21.1".into();
    assert_eq!(req.name, "yarn");
    assert_eq!(req.version, Some("1.21.1".to_string()));
}

#[test]
fn test_from_string() {
    let req: RuntimeRequest = String::from("node@20").into();
    assert_eq!(req.name, "node");
    assert_eq!(req.version, Some("20".to_string()));
}

// --- executable override ---

#[test]
fn test_parse_executable_override() {
    let req = RuntimeRequest::parse("msvc::cl");
    assert_eq!(req.name, "msvc");
    assert_eq!(req.executable, Some("cl".to_string()));
    assert_eq!(req.version, None);
    assert_eq!(req.shell, None);
}

#[test]
fn test_parse_executable_override_with_version() {
    let req = RuntimeRequest::parse("msvc@14.42::cl");
    assert_eq!(req.name, "msvc");
    assert_eq!(req.executable, Some("cl".to_string()));
    assert_eq!(req.version, Some("14.42".to_string()));
}

#[test]
fn test_parse_executable_override_empty_exe() {
    let req = RuntimeRequest::parse("msvc@14.42::");
    assert_eq!(req.name, "msvc");
    assert_eq!(req.executable, None);
    assert_eq!(req.version, Some("14.42".to_string()));
}

#[test]
fn test_parse_executable_override_empty_all() {
    let req = RuntimeRequest::parse("msvc::");
    assert_eq!(req.name, "msvc");
    assert_eq!(req.executable, None);
    assert_eq!(req.version, None);
}

#[test]
fn test_display_with_executable_override() {
    // Display outputs canonical format: runtime@version::executable
    let req = RuntimeRequest {
        name: "msvc".to_string(),
        executable: Some("cl".to_string()),
        version: Some("14.42".to_string()),
        shell: None,
    };
    assert_eq!(format!("{}", req), "msvc@14.42::cl");

    let req = RuntimeRequest {
        name: "msvc".to_string(),
        executable: Some("cl".to_string()),
        version: None,
        shell: None,
    };
    assert_eq!(format!("{}", req), "msvc::cl");
}

// --- canonical format: runtime@version::executable ---

#[test]
fn test_parse_canonical_version_before_exe() {
    // Canonical: runtime@version::executable
    let req = RuntimeRequest::parse("msvc@14.19::cl");
    assert_eq!(req.name, "msvc");
    assert_eq!(req.executable, Some("cl".to_string()));
    assert_eq!(req.version, Some("14.19".to_string()));
    assert_eq!(req.shell, None);
}

#[test]
fn test_parse_canonical_version_before_exe_complex() {
    // Canonical with full semver
    let req = RuntimeRequest::parse("msvc@14.42.0::link");
    assert_eq!(req.name, "msvc");
    assert_eq!(req.executable, Some("link".to_string()));
    assert_eq!(req.version, Some("14.42.0".to_string()));
}

#[test]
fn test_parse_canonical_version_before_shell() {
    // Canonical with shell: runtime@version::shell
    let req = RuntimeRequest::parse("node@22::powershell");
    assert_eq!(req.name, "node");
    assert_eq!(req.shell, Some("powershell".to_string()));
    assert_eq!(req.version, Some("22".to_string()));
    assert!(req.is_shell_request());
}

#[test]
fn test_parse_noncanonical_version_after_exe_is_not_supported() {
    let req = RuntimeRequest::parse("msvc::cl@14.42");
    assert_eq!(req.name, "msvc");
    assert_eq!(req.executable, Some("cl@14.42".to_string()));
    assert_eq!(req.version, None);
}

#[test]
fn test_parse_canonical_roundtrip() {
    // Parse canonical format, display should produce canonical format
    let req = RuntimeRequest::parse("msvc@14.42::cl");
    assert_eq!(format!("{}", req), "msvc@14.42::cl");
}

#[test]
fn test_parse_compatibility_display_canonical() {
    // Parse compatibility format, display should produce canonical format
    let req = RuntimeRequest::parse("msvc::cl@14.42");
    assert_eq!(format!("{}", req), "msvc@14.42::cl");
}

// --- shell syntax (runtime::shell) ---

#[test]
fn test_parse_git_bash_shell() {
    let req = RuntimeRequest::parse("git::git-bash");
    assert_eq!(req.name, "git");
    assert_eq!(req.shell, Some("git-bash".to_string()));
    assert_eq!(req.executable, None);
    assert_eq!(req.version, None);
    assert!(req.is_shell_request());
    assert_eq!(req.shell_name(), Some("git-bash"));
}

#[test]
fn test_parse_cmd_shell() {
    let req = RuntimeRequest::parse("git::cmd");
    assert_eq!(req.name, "git");
    assert_eq!(req.shell, Some("cmd".to_string()));
    assert_eq!(req.executable, None);
    assert!(req.is_shell_request());
}

#[test]
fn test_parse_powershell_shell() {
    let req = RuntimeRequest::parse("node::powershell");
    assert_eq!(req.name, "node");
    assert_eq!(req.shell, Some("powershell".to_string()));
    assert!(req.is_shell_request());
}

#[test]
fn test_parse_bash_shell() {
    let req = RuntimeRequest::parse("go::bash");
    assert_eq!(req.name, "go");
    assert_eq!(req.shell, Some("bash".to_string()));
    assert!(req.is_shell_request());
}

#[test]
fn test_parse_shell_with_version() {
    let req = RuntimeRequest::parse("git@2.43::git-bash");
    assert_eq!(req.name, "git");
    assert_eq!(req.shell, Some("git-bash".to_string()));
    assert_eq!(req.version, Some("2.43".to_string()));
    assert!(req.is_shell_request());
}

#[test]
fn test_executable_vs_shell_distinction() {
    // "cl" is NOT a known shell → executable
    let req = RuntimeRequest::parse("msvc::cl");
    assert_eq!(req.executable, Some("cl".to_string()));
    assert_eq!(req.shell, None);
    assert!(!req.is_shell_request());

    // "cmd" IS a known shell
    let req = RuntimeRequest::parse("msvc::cmd");
    assert_eq!(req.executable, None);
    assert_eq!(req.shell, Some("cmd".to_string()));
    assert!(req.is_shell_request());
}

#[test]
fn test_display_with_shell() {
    // Display outputs canonical format: runtime@version::shell
    let req = RuntimeRequest {
        name: "git".to_string(),
        shell: Some("git-bash".to_string()),
        version: Some("2.43".to_string()),
        executable: None,
    };
    assert_eq!(format!("{}", req), "git@2.43::git-bash");

    let req = RuntimeRequest {
        name: "git".to_string(),
        shell: Some("git-bash".to_string()),
        version: None,
        executable: None,
    };
    assert_eq!(format!("{}", req), "git::git-bash");
}
