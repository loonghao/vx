//! Dependencies configuration tests
//!
//! Tests for dependency management configuration parsing.

use rstest::rstest;
use vx_config::parse_config_str;

// ============================================
// Dependencies Config Parsing Tests
// ============================================

#[test]
fn test_parse_dependencies_config_basic() {
    let content = r#"
[dependencies]
lockfile = true
audit = true
auto_update = "patch"
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();
    assert_eq!(deps.lockfile, Some(true));
    assert_eq!(deps.audit, Some(true));
    assert_eq!(deps.auto_update, Some("patch".to_string()));
}

#[rstest]
#[case("none")]
#[case("patch")]
#[case("minor")]
#[case("major")]
fn test_parse_auto_update_strategies(#[case] strategy: &str) {
    let content = format!(
        r#"
[dependencies]
auto_update = "{}"
"#,
        strategy
    );
    let config = parse_config_str(&content).unwrap();
    let deps = config.dependencies.unwrap();
    assert_eq!(deps.auto_update, Some(strategy.to_string()));
}

#[test]
fn test_parse_dependencies_lockfile_disabled() {
    let content = r#"
[dependencies]
lockfile = false
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();
    assert_eq!(deps.lockfile, Some(false));
}

#[test]
fn test_parse_dependencies_audit_disabled() {
    let content = r#"
[dependencies]
audit = false
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();
    assert_eq!(deps.audit, Some(false));
}

#[test]
fn test_parse_node_dependencies_config() {
    let content = r#"
[dependencies.node]
package_manager = "pnpm"
registry = "https://registry.npmmirror.com"
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();
    let node = deps.node.unwrap();

    assert_eq!(node.package_manager, Some("pnpm".to_string()));
    assert_eq!(
        node.registry,
        Some("https://registry.npmmirror.com".to_string())
    );
}

#[rstest]
#[case("npm")]
#[case("yarn")]
#[case("pnpm")]
#[case("bun")]
fn test_parse_node_package_managers(#[case] pm: &str) {
    let content = format!(
        r#"
[dependencies.node]
package_manager = "{}"
"#,
        pm
    );
    let config = parse_config_str(&content).unwrap();
    let deps = config.dependencies.unwrap();
    let node = deps.node.unwrap();

    assert_eq!(node.package_manager, Some(pm.to_string()));
}

#[test]
fn test_parse_python_dependencies_config() {
    let content = r#"
[dependencies.python]
index_url = "https://pypi.tuna.tsinghua.edu.cn/simple"
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();
    let python = deps.python.unwrap();

    assert_eq!(
        python.index_url,
        Some("https://pypi.tuna.tsinghua.edu.cn/simple".to_string())
    );
}

#[test]
fn test_parse_dependencies_constraints() {
    let content = r#"
[dependencies.constraints]
node = ">=18.0.0"
python = ">=3.10"
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();

    assert!(!deps.constraints.is_empty());
    assert!(deps.constraints.contains_key("node"));
    assert!(deps.constraints.contains_key("python"));
}

#[test]
fn test_parse_full_dependencies_config() {
    let content = r#"
[dependencies]
lockfile = true
audit = true
auto_update = "minor"

[dependencies.node]
package_manager = "pnpm"
registry = "https://registry.npmjs.org"

[dependencies.python]
index_url = "https://pypi.org/simple"

[dependencies.constraints]
node = ">=18.0.0"
python = ">=3.10"
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();

    assert_eq!(deps.lockfile, Some(true));
    assert_eq!(deps.audit, Some(true));
    assert_eq!(deps.auto_update, Some("minor".to_string()));
    assert!(deps.node.is_some());
    assert!(deps.python.is_some());
    assert!(!deps.constraints.is_empty());
}

#[test]
fn test_dependencies_config_empty() {
    let content = r#"
[dependencies]
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();

    assert!(deps.lockfile.is_none());
    assert!(deps.audit.is_none());
    assert!(deps.auto_update.is_none());
    assert!(deps.node.is_none());
    assert!(deps.python.is_none());
}

#[test]
fn test_parse_python_with_extra_index() {
    let content = r#"
[dependencies.python]
index_url = "https://pypi.org/simple"
extra_index_urls = ["https://pypi.tuna.tsinghua.edu.cn/simple"]
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();
    let python = deps.python.unwrap();

    assert_eq!(
        python.extra_index_urls,
        vec!["https://pypi.tuna.tsinghua.edu.cn/simple".to_string()]
    );
}

// ============================================
// Go Dependencies Config Tests
// ============================================

#[test]
fn test_parse_go_dependencies_config() {
    let content = r#"
[dependencies.go]
proxy = "https://goproxy.cn,direct"
private = "github.com/mycompany/*"
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();
    let go = deps.go.unwrap();

    assert_eq!(go.proxy, Some("https://goproxy.cn,direct".to_string()));
    assert_eq!(go.private, Some("github.com/mycompany/*".to_string()));
}

#[test]
fn test_parse_go_dependencies_full() {
    let content = r#"
[dependencies.go]
proxy = "https://goproxy.io,direct"
private = "github.com/private/*,gitlab.com/internal/*"
sumdb = "sum.golang.org"
nosumdb = "github.com/private/*"
vendor = true
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();
    let go = deps.go.unwrap();

    assert_eq!(go.proxy, Some("https://goproxy.io,direct".to_string()));
    assert_eq!(
        go.private,
        Some("github.com/private/*,gitlab.com/internal/*".to_string())
    );
    assert_eq!(go.sumdb, Some("sum.golang.org".to_string()));
    assert_eq!(go.nosumdb, Some("github.com/private/*".to_string()));
    assert_eq!(go.vendor, Some(true));
}

#[test]
fn test_parse_go_mod_mode() {
    let content = r#"
[dependencies.go]
mod_mode = "readonly"
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();
    let go = deps.go.unwrap();

    assert_eq!(go.mod_mode, Some("readonly".to_string()));
}

#[rstest]
#[case("https://goproxy.cn,direct")]
#[case("https://proxy.golang.org,direct")]
#[case("https://goproxy.io,direct")]
#[case("https://mirrors.aliyun.com/goproxy/,direct")]
fn test_parse_go_proxy_presets(#[case] proxy: &str) {
    let content = format!(
        r#"
[dependencies.go]
proxy = "{}"
"#,
        proxy
    );
    let config = parse_config_str(&content).unwrap();
    let deps = config.dependencies.unwrap();
    let go = deps.go.unwrap();

    assert_eq!(go.proxy, Some(proxy.to_string()));
}

// ============================================
// C++ Dependencies Config Tests
// ============================================

#[test]
fn test_parse_cpp_dependencies_vcpkg() {
    let content = r#"
[dependencies.cpp]
package_manager = "vcpkg"
vcpkg_root = "/opt/vcpkg"
vcpkg_triplet = "x64-linux"
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();
    let cpp = deps.cpp.unwrap();

    assert_eq!(cpp.package_manager, Some("vcpkg".to_string()));
    assert_eq!(cpp.vcpkg_root, Some("/opt/vcpkg".to_string()));
    assert_eq!(cpp.vcpkg_triplet, Some("x64-linux".to_string()));
}

#[test]
fn test_parse_cpp_dependencies_conan() {
    let content = r#"
[dependencies.cpp]
package_manager = "conan"
conan_remote = "https://center.conan.io"
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();
    let cpp = deps.cpp.unwrap();

    assert_eq!(cpp.package_manager, Some("conan".to_string()));
    assert_eq!(
        cpp.conan_remote,
        Some("https://center.conan.io".to_string())
    );
}

#[test]
fn test_parse_cpp_dependencies_cmake() {
    let content = r#"
[dependencies.cpp]
package_manager = "cmake"
cmake_generator = "Ninja"
cmake_build_type = "Release"
std = "17"
compiler = "clang"
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();
    let cpp = deps.cpp.unwrap();

    assert_eq!(cpp.package_manager, Some("cmake".to_string()));
    assert_eq!(cpp.cmake_generator, Some("Ninja".to_string()));
    assert_eq!(cpp.cmake_build_type, Some("Release".to_string()));
    assert_eq!(cpp.std, Some("17".to_string()));
    assert_eq!(cpp.compiler, Some("clang".to_string()));
}

#[test]
fn test_parse_cpp_cmake_options() {
    let content = r#"
[dependencies.cpp]
cmake_generator = "Ninja"

[dependencies.cpp.cmake_options]
BUILD_TESTS = "ON"
BUILD_EXAMPLES = "OFF"
CMAKE_EXPORT_COMPILE_COMMANDS = "ON"
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();
    let cpp = deps.cpp.unwrap();

    assert_eq!(
        cpp.cmake_options.get("BUILD_TESTS"),
        Some(&"ON".to_string())
    );
    assert_eq!(
        cpp.cmake_options.get("BUILD_EXAMPLES"),
        Some(&"OFF".to_string())
    );
    assert_eq!(
        cpp.cmake_options.get("CMAKE_EXPORT_COMPILE_COMMANDS"),
        Some(&"ON".to_string())
    );
}

#[rstest]
#[case("vcpkg")]
#[case("conan")]
#[case("cmake")]
fn test_parse_cpp_package_managers(#[case] pm: &str) {
    let content = format!(
        r#"
[dependencies.cpp]
package_manager = "{}"
"#,
        pm
    );
    let config = parse_config_str(&content).unwrap();
    let deps = config.dependencies.unwrap();
    let cpp = deps.cpp.unwrap();

    assert_eq!(cpp.package_manager, Some(pm.to_string()));
}

#[rstest]
#[case("x64-windows")]
#[case("x64-linux")]
#[case("x64-osx")]
#[case("arm64-osx")]
fn test_parse_vcpkg_triplets(#[case] triplet: &str) {
    let content = format!(
        r#"
[dependencies.cpp]
vcpkg_triplet = "{}"
"#,
        triplet
    );
    let config = parse_config_str(&content).unwrap();
    let deps = config.dependencies.unwrap();
    let cpp = deps.cpp.unwrap();

    assert_eq!(cpp.vcpkg_triplet, Some(triplet.to_string()));
}

#[rstest]
#[case("11")]
#[case("14")]
#[case("17")]
#[case("20")]
#[case("23")]
fn test_parse_cpp_standards(#[case] std: &str) {
    let content = format!(
        r#"
[dependencies.cpp]
std = "{}"
"#,
        std
    );
    let config = parse_config_str(&content).unwrap();
    let deps = config.dependencies.unwrap();
    let cpp = deps.cpp.unwrap();

    assert_eq!(cpp.std, Some(std.to_string()));
}

// ============================================
// Full Dependencies Config with All Languages
// ============================================

#[test]
fn test_parse_full_dependencies_with_all_languages() {
    let content = r#"
[dependencies]
lockfile = true
audit = true
auto_update = "minor"

[dependencies.node]
package_manager = "pnpm"
registry = "https://registry.npmjs.org"

[dependencies.python]
index_url = "https://pypi.org/simple"
extra_index_urls = ["https://pypi.tuna.tsinghua.edu.cn/simple"]

[dependencies.go]
proxy = "https://goproxy.cn,direct"
private = "github.com/mycompany/*"

[dependencies.cpp]
package_manager = "cmake"
cmake_generator = "Ninja"
std = "17"

[dependencies.constraints]
node = ">=18.0.0"
python = ">=3.10"
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.unwrap();

    assert_eq!(deps.lockfile, Some(true));
    assert_eq!(deps.audit, Some(true));
    assert!(deps.node.is_some());
    assert!(deps.python.is_some());
    assert!(deps.go.is_some());
    assert!(deps.cpp.is_some());

    let go = deps.go.unwrap();
    assert_eq!(go.proxy, Some("https://goproxy.cn,direct".to_string()));

    let cpp = deps.cpp.unwrap();
    assert_eq!(cpp.cmake_generator, Some("Ninja".to_string()));
}
