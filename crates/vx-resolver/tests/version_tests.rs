//! Version solver tests

use rstest::rstest;
use vx_resolver::version::{
    RangeConstraint, RangeOp, ResolvedVersion, SolverConfig, SolverStatus, VersionConstraint,
    VersionRequest, VersionSolver,
};
use vx_resolver::Ecosystem;
use vx_runtime::VersionInfo;

mod constraint_tests {
    use super::*;
    use vx_resolver::version::Version;

    #[rstest]
    #[case("1.2.3", 1, 2, 3, None)]
    #[case("v1.2.3", 1, 2, 3, None)]
    #[case("1.2", 1, 2, 0, None)]
    #[case("1", 1, 0, 0, None)]
    #[case("20.10.0", 20, 10, 0, None)]
    #[case("3.11.11", 3, 11, 11, None)]
    #[case("1.0.0-alpha", 1, 0, 0, Some("alpha"))]
    #[case("1.0.0-beta.1", 1, 0, 0, Some("beta.1"))]
    fn test_version_parse(
        #[case] input: &str,
        #[case] major: u32,
        #[case] minor: u32,
        #[case] patch: u32,
        #[case] prerelease: Option<&str>,
    ) {
        let version = Version::parse(input).expect("Should parse");
        assert_eq!(version.major, major);
        assert_eq!(version.minor, minor);
        assert_eq!(version.patch, patch);
        assert_eq!(version.prerelease.as_deref(), prerelease);
    }

    #[rstest]
    #[case("2.0.0", "1.0.0", std::cmp::Ordering::Greater)]
    #[case("1.1.0", "1.0.0", std::cmp::Ordering::Greater)]
    #[case("1.0.1", "1.0.0", std::cmp::Ordering::Greater)]
    #[case("1.0.0", "1.0.0", std::cmp::Ordering::Equal)]
    #[case("1.0.0", "1.0.0-alpha", std::cmp::Ordering::Greater)]
    #[case("1.0.0-beta", "1.0.0-alpha", std::cmp::Ordering::Greater)]
    fn test_version_ordering(
        #[case] a: &str,
        #[case] b: &str,
        #[case] expected: std::cmp::Ordering,
    ) {
        let va = Version::parse(a).unwrap();
        let vb = Version::parse(b).unwrap();
        assert_eq!(va.cmp(&vb), expected);
    }

    #[rstest]
    #[case(RangeOp::Ge, "1.0.0", "1.0.0", true)]
    #[case(RangeOp::Ge, "1.0.0", "1.0.1", true)]
    #[case(RangeOp::Ge, "1.0.0", "0.9.0", false)]
    #[case(RangeOp::Lt, "2.0.0", "1.9.9", true)]
    #[case(RangeOp::Lt, "2.0.0", "2.0.0", false)]
    #[case(RangeOp::Tilde, "1.2.0", "1.2.5", true)]
    #[case(RangeOp::Tilde, "1.2.0", "1.3.0", false)]
    #[case(RangeOp::Caret, "1.0.0", "1.9.9", true)]
    #[case(RangeOp::Caret, "1.0.0", "2.0.0", false)]
    fn test_range_constraint(
        #[case] op: RangeOp,
        #[case] constraint_version: &str,
        #[case] test_version: &str,
        #[case] expected: bool,
    ) {
        let cv = Version::parse(constraint_version).unwrap();
        let tv = Version::parse(test_version).unwrap();
        let constraint = RangeConstraint::new(op, cv);
        assert_eq!(constraint.satisfies(&tv), expected);
    }
}

mod request_tests {
    use super::*;

    #[rstest]
    #[case("latest", true)]
    #[case("stable", true)]
    #[case("1.0.0", false)]
    #[case("^1.0.0", false)]
    fn test_is_latest(#[case] input: &str, #[case] expected: bool) {
        let request = VersionRequest::parse(input);
        assert_eq!(request.is_latest(), expected);
    }

    #[rstest]
    #[case("1.0.0", true)]
    #[case("1.2.3", true)]
    #[case("1.0", false)]
    #[case("latest", false)]
    fn test_is_exact(#[case] input: &str, #[case] expected: bool) {
        let request = VersionRequest::parse(input);
        assert_eq!(request.is_exact(), expected);
    }

    #[rstest]
    #[case("3.11", true)]
    #[case("20", true)]
    #[case("3.11.0", false)]
    #[case("latest", false)]
    fn test_is_partial(#[case] input: &str, #[case] expected: bool) {
        let request = VersionRequest::parse(input);
        assert_eq!(request.is_partial(), expected);
    }

    #[test]
    fn test_parse_range_constraint() {
        let request = VersionRequest::parse(">=3.9,<3.12");
        if let VersionConstraint::Range(constraints) = &request.constraint {
            assert_eq!(constraints.len(), 2);
        } else {
            panic!("Expected Range constraint");
        }
    }

    #[test]
    fn test_parse_caret_constraint() {
        let request = VersionRequest::parse("^1.2.3");
        assert!(matches!(request.constraint, VersionConstraint::Caret(_)));
    }

    #[test]
    fn test_parse_tilde_constraint() {
        let request = VersionRequest::parse("~1.2.3");
        assert!(matches!(request.constraint, VersionConstraint::Tilde(_)));
    }

    #[test]
    fn test_parse_wildcard() {
        let request = VersionRequest::parse("3.11.*");
        assert!(matches!(
            request.constraint,
            VersionConstraint::Wildcard {
                major: 3,
                minor: 11
            }
        ));
    }
}

mod solver_tests {
    use super::*;

    fn make_versions(versions: &[&str]) -> Vec<VersionInfo> {
        versions.iter().map(|v| VersionInfo::new(*v)).collect()
    }

    fn make_versions_with_lts(versions: &[(&str, bool)]) -> Vec<VersionInfo> {
        versions
            .iter()
            .map(|(v, lts)| VersionInfo::new(*v).with_lts(*lts))
            .collect()
    }

    #[test]
    fn test_solver_resolve_latest() {
        let solver = VersionSolver::new();
        let available = make_versions(&["1.0.0", "1.1.0", "2.0.0"]);
        let request = VersionRequest::parse("latest");

        let result = solver.resolve("test", &request, &available, &Ecosystem::Node);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version_string(), "2.0.0");
    }

    #[test]
    fn test_solver_resolve_partial_python() {
        let solver = VersionSolver::new();
        let available = make_versions(&["3.10.0", "3.11.0", "3.11.5", "3.11.11", "3.12.0"]);
        let request = VersionRequest::parse("3.11");

        let result = solver.resolve("python", &request, &available, &Ecosystem::Python);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version_string(), "3.11.11");
    }

    #[test]
    fn test_solver_resolve_major_only() {
        let solver = VersionSolver::new();
        let available = make_versions(&["18.0.0", "18.10.0", "20.0.0", "20.10.0", "22.0.0"]);
        let request = VersionRequest::parse("20");

        let result = solver.resolve("node", &request, &available, &Ecosystem::Node);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version_string(), "20.10.0");
    }

    #[test]
    fn test_solver_resolve_exact() {
        let solver = VersionSolver::new();
        let available = make_versions(&["1.0.0", "1.1.0", "2.0.0"]);
        let request = VersionRequest::parse("1.1.0");

        let result = solver.resolve("test", &request, &available, &Ecosystem::Node);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version_string(), "1.1.0");
    }

    #[test]
    fn test_solver_resolve_range() {
        let solver = VersionSolver::new();
        let available = make_versions(&["3.8.0", "3.9.0", "3.10.0", "3.11.0", "3.12.0"]);
        let request = VersionRequest::parse(">=3.9,<3.12");

        let result = solver.resolve("python", &request, &available, &Ecosystem::Python);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version_string(), "3.11.0");
    }

    #[test]
    fn test_solver_resolve_caret() {
        let solver = VersionSolver::new();
        let available = make_versions(&["1.0.0", "1.1.0", "1.9.0", "2.0.0"]);
        let request = VersionRequest::parse("^1.0.0");

        let result = solver.resolve("test", &request, &available, &Ecosystem::Node);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version_string(), "1.9.0");
    }

    #[test]
    fn test_solver_resolve_tilde() {
        let solver = VersionSolver::new();
        let available = make_versions(&["1.2.0", "1.2.5", "1.2.9", "1.3.0"]);
        let request = VersionRequest::parse("~1.2.0");

        let result = solver.resolve("test", &request, &available, &Ecosystem::Node);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version_string(), "1.2.9");
    }

    #[test]
    fn test_solver_resolve_not_found() {
        let solver = VersionSolver::new();
        let available = make_versions(&["1.0.0", "1.1.0"]);
        let request = VersionRequest::parse("2.0.0");

        let result = solver.resolve("test", &request, &available, &Ecosystem::Node);
        assert!(result.is_err());
    }

    #[test]
    fn test_solver_prefer_lts() {
        let config = SolverConfig {
            prefer_lts: true,
            ..Default::default()
        };
        let solver = VersionSolver::with_config(config);

        let available =
            make_versions_with_lts(&[("18.0.0", true), ("20.0.0", true), ("22.0.0", false)]);
        let request = VersionRequest::parse("latest");

        let result = solver.resolve("node", &request, &available, &Ecosystem::Node);
        assert!(result.is_ok());
        // Should prefer LTS version 20.0.0 over non-LTS 22.0.0
        assert_eq!(result.unwrap().version_string(), "20.0.0");
    }

    #[test]
    fn test_solver_exclude_prerelease() {
        let solver = VersionSolver::new();
        let available = vec![
            VersionInfo::new("1.0.0"),
            VersionInfo::new("2.0.0"),
            VersionInfo::new("3.0.0-alpha").with_prerelease(true),
        ];
        let request = VersionRequest::parse("latest");

        let result = solver.resolve("test", &request, &available, &Ecosystem::Node);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version_string(), "2.0.0");
    }

    #[test]
    fn test_solver_resolve_all() {
        let solver = VersionSolver::new();

        let requests = vec![
            (
                "node".to_string(),
                VersionRequest::parse("20"),
                Ecosystem::Node,
                make_versions(&["18.0.0", "20.0.0", "20.10.0", "22.0.0"]),
            ),
            (
                "python".to_string(),
                VersionRequest::parse("3.11"),
                Ecosystem::Python,
                make_versions(&["3.10.0", "3.11.0", "3.11.11", "3.12.0"]),
            ),
        ];

        let result = solver.resolve_all(&requests);
        assert!(result.is_success());
        assert_eq!(result.status, SolverStatus::Solved);
        assert_eq!(result.resolved.len(), 2);
        assert_eq!(result.get("node").unwrap().version_string(), "20.10.0");
        assert_eq!(result.get("python").unwrap().version_string(), "3.11.11");
    }

    #[test]
    fn test_version_satisfies() {
        let solver = VersionSolver::new();

        // Caret
        assert!(solver.version_satisfies("1.2.3", "^1.0.0", &Ecosystem::Node));
        assert!(solver.version_satisfies("1.9.9", "^1.0.0", &Ecosystem::Node));
        assert!(!solver.version_satisfies("2.0.0", "^1.0.0", &Ecosystem::Node));

        // Partial
        assert!(solver.version_satisfies("3.11.5", "3.11", &Ecosystem::Python));
        assert!(solver.version_satisfies("3.11.11", "3.11", &Ecosystem::Python));
        assert!(!solver.version_satisfies("3.12.0", "3.11", &Ecosystem::Python));

        // Range
        assert!(solver.version_satisfies("3.10.0", ">=3.9,<3.12", &Ecosystem::Python));
        assert!(!solver.version_satisfies("3.8.0", ">=3.9,<3.12", &Ecosystem::Python));
        assert!(!solver.version_satisfies("3.12.0", ">=3.9,<3.12", &Ecosystem::Python));
    }

    #[test]
    fn test_resolve_version_string() {
        let solver = VersionSolver::new();
        let available = make_versions(&["3.10.0", "3.11.0", "3.11.11", "3.12.0"]);

        let result =
            solver.resolve_version_string("python", "3.11", &available, &Ecosystem::Python);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "3.11.11");
    }
}

mod resolved_version_tests {
    use super::*;
    use vx_resolver::version::Version;

    #[test]
    fn test_resolved_version_metadata() {
        let version = Version::new(3, 11, 11);
        let resolved = ResolvedVersion::new(version, "3.11")
            .with_source("python-build-standalone")
            .with_metadata("release_date", "20251217")
            .with_metadata("checksum", "sha256:abc123");

        assert_eq!(resolved.version_string(), "3.11.11");
        assert_eq!(resolved.resolved_from, "3.11");
        assert_eq!(resolved.source, "python-build-standalone");
        assert_eq!(
            resolved.get_metadata("release_date"),
            Some(&"20251217".to_string())
        );
        assert_eq!(
            resolved.get_metadata("checksum"),
            Some(&"sha256:abc123".to_string())
        );
    }
}

mod lockfile_tests {
    use super::*;
    use std::collections::HashMap;
    use vx_resolver::version::{LockFile, LockFileInconsistency, LockedTool};

    #[test]
    fn test_lockfile_new() {
        let lockfile = LockFile::new();
        assert_eq!(lockfile.version, LockFile::current_version());
        assert!(lockfile.tools.is_empty());
        assert!(lockfile.dependencies.is_empty());
    }

    #[test]
    fn test_lock_tool() {
        let mut lockfile = LockFile::new();
        let tool = LockedTool::new("3.11.11", "python-build-standalone")
            .with_resolved_from("3.11")
            .with_ecosystem(Ecosystem::Python);

        lockfile.lock_tool("python", tool);

        assert!(lockfile.is_locked("python"));
        assert!(!lockfile.is_locked("node"));

        let locked = lockfile.get_tool("python").unwrap();
        assert_eq!(locked.version, "3.11.11");
        assert_eq!(locked.resolved_from, "3.11");
        assert_eq!(locked.source, "python-build-standalone");
        assert_eq!(locked.ecosystem, Ecosystem::Python);
    }

    #[test]
    fn test_unlock_tool() {
        let mut lockfile = LockFile::new();
        lockfile.lock_tool("python", LockedTool::new("3.11.11", "test"));

        assert!(lockfile.is_locked("python"));

        let removed = lockfile.unlock_tool("python");
        assert!(removed.is_some());
        assert!(!lockfile.is_locked("python"));
    }

    #[test]
    fn test_tool_names() {
        let mut lockfile = LockFile::new();
        lockfile.lock_tool("python", LockedTool::new("3.11.11", "test"));
        lockfile.lock_tool("node", LockedTool::new("20.18.0", "test"));

        let names = lockfile.tool_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"python"));
        assert!(names.contains(&"node"));
    }

    #[test]
    fn test_add_dependencies() {
        let mut lockfile = LockFile::new();
        lockfile.lock_tool("node", LockedTool::new("20.18.0", "test"));
        lockfile.lock_tool("npm", LockedTool::new("10.0.0", "test"));

        lockfile.add_dependency("npm", vec!["node".to_string()]);

        let deps = lockfile.get_dependencies("npm").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], "node");
    }

    #[test]
    fn test_lockfile_serialize_parse() {
        let mut lockfile = LockFile::new();
        lockfile.lock_tool(
            "python",
            LockedTool::new("3.11.11", "python-build-standalone").with_resolved_from("3.11"),
        );
        lockfile.lock_tool(
            "node",
            LockedTool::new("20.18.0", "nodejs.org").with_resolved_from("20"),
        );

        let content = lockfile.to_string().unwrap();
        assert!(content.contains("vx.lock"));
        assert!(content.contains("[tools.python]"));
        assert!(content.contains("version = \"3.11.11\""));

        // Parse back
        let parsed = LockFile::parse(&content).unwrap();
        assert_eq!(parsed.tools.len(), 2);
        assert_eq!(parsed.get_tool("python").unwrap().version, "3.11.11");
        assert_eq!(parsed.get_tool("node").unwrap().version, "20.18.0");
    }

    #[test]
    fn test_lockfile_parse_format() {
        let content = r#"
version = 1

[metadata]
generated_at = "2025-12-30T10:00:00Z"
vx_version = "0.7.0"
platform = "x86_64-pc-windows-msvc"

[tools.python]
version = "3.11.11"
source = "python-build-standalone"
resolved_from = "3.11"

[tools.node]
version = "20.18.0"
source = "nodejs.org"
resolved_from = "20"

[dependencies]
npm = ["node"]
"#;

        let lockfile = LockFile::parse(content).unwrap();
        assert_eq!(lockfile.version, 1);
        assert_eq!(lockfile.tools.len(), 2);
        assert_eq!(lockfile.get_tool("python").unwrap().version, "3.11.11");
        assert_eq!(lockfile.get_tool("node").unwrap().version, "20.18.0");
        assert_eq!(
            lockfile.get_dependencies("npm"),
            Some(&vec!["node".to_string()])
        );
    }

    #[test]
    fn test_check_consistency_all_match() {
        let mut lockfile = LockFile::new();
        lockfile.lock_tool(
            "python",
            LockedTool::new("3.11.11", "test").with_resolved_from("3.11"),
        );
        lockfile.lock_tool(
            "node",
            LockedTool::new("20.18.0", "test").with_resolved_from("20"),
        );

        let mut config = HashMap::new();
        config.insert("python".to_string(), "3.11".to_string());
        config.insert("node".to_string(), "20".to_string());

        let inconsistencies = lockfile.check_consistency(&config);
        assert!(inconsistencies.is_empty());
    }

    #[test]
    fn test_check_consistency_missing_in_lock() {
        let mut lockfile = LockFile::new();
        lockfile.lock_tool(
            "python",
            LockedTool::new("3.11.11", "test").with_resolved_from("3.11"),
        );

        let mut config = HashMap::new();
        config.insert("python".to_string(), "3.11".to_string());
        config.insert("node".to_string(), "20".to_string());

        let inconsistencies = lockfile.check_consistency(&config);
        assert_eq!(inconsistencies.len(), 1);
        assert!(matches!(
            &inconsistencies[0],
            LockFileInconsistency::MissingInLock { tool, .. } if tool == "node"
        ));
    }

    #[test]
    fn test_check_consistency_extra_in_lock() {
        let mut lockfile = LockFile::new();
        lockfile.lock_tool(
            "python",
            LockedTool::new("3.11.11", "test").with_resolved_from("3.11"),
        );
        lockfile.lock_tool(
            "node",
            LockedTool::new("20.18.0", "test").with_resolved_from("20"),
        );

        let mut config = HashMap::new();
        config.insert("python".to_string(), "3.11".to_string());

        let inconsistencies = lockfile.check_consistency(&config);
        assert_eq!(inconsistencies.len(), 1);
        assert!(matches!(
            &inconsistencies[0],
            LockFileInconsistency::ExtraInLock { tool } if tool == "node"
        ));
    }

    #[test]
    fn test_check_consistency_version_mismatch() {
        let mut lockfile = LockFile::new();
        lockfile.lock_tool(
            "python",
            LockedTool::new("3.11.11", "test").with_resolved_from("3.11"),
        );

        let mut config = HashMap::new();
        config.insert("python".to_string(), "3.12".to_string()); // Changed version

        let inconsistencies = lockfile.check_consistency(&config);
        assert_eq!(inconsistencies.len(), 1);
        assert!(matches!(
            &inconsistencies[0],
            LockFileInconsistency::VersionMismatch { tool, config_version, locked_from }
                if tool == "python" && config_version == "3.12" && locked_from == "3.11"
        ));
    }

    #[test]
    fn test_lockfile_merge() {
        let mut lockfile1 = LockFile::new();
        lockfile1.lock_tool("python", LockedTool::new("3.11.11", "test"));

        let mut lockfile2 = LockFile::new();
        lockfile2.lock_tool("node", LockedTool::new("20.18.0", "test"));
        lockfile2.lock_tool("python", LockedTool::new("3.12.0", "test")); // Override

        lockfile1.merge(&lockfile2);

        assert_eq!(lockfile1.tools.len(), 2);
        assert_eq!(lockfile1.get_tool("python").unwrap().version, "3.12.0"); // Overwritten
        assert_eq!(lockfile1.get_tool("node").unwrap().version, "20.18.0");
    }

    #[test]
    fn test_locked_tool_with_metadata() {
        let tool = LockedTool::new("3.11.11", "python-build-standalone")
            .with_resolved_from("3.11")
            .with_ecosystem(Ecosystem::Python)
            .with_checksum("sha256:abc123")
            .with_metadata("release_date", "20251217");

        assert_eq!(tool.version, "3.11.11");
        assert_eq!(tool.source, "python-build-standalone");
        assert_eq!(tool.resolved_from, "3.11");
        assert_eq!(tool.ecosystem, Ecosystem::Python);
        assert_eq!(tool.checksum, Some("sha256:abc123".to_string()));
        assert_eq!(
            tool.metadata.get("release_date"),
            Some(&"20251217".to_string())
        );
    }

    #[test]
    fn test_locked_tool_parsed_version() {
        let tool = LockedTool::new("3.11.11", "test");
        let version = tool.parsed_version().unwrap();
        assert_eq!(version.major, 3);
        assert_eq!(version.minor, 11);
        assert_eq!(version.patch, 11);
    }
}
