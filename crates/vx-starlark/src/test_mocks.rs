//! Standard mocks for Starlark provider tests
//!
//! This module provides pre-built mock implementations for `@vx//stdlib/*`
//! modules used in provider.star files. Use these to set up test environments
//! without duplicating mock code across provider tests.
//!
//! # Example
//!
//! ```rust
//! use starlark::assert::Assert;
//! use vx_starlark::test_mocks::setup_provider_test_mocks;
//!
//! let mut a = Assert::new();
//! setup_provider_test_mocks(&mut a);
//! a.module("provider.star", PROVIDER_STAR);
//! ```

use starlark::assert::Assert;

/// Standard mock for @vx//stdlib:env.star
pub const MOCK_ENV_STAR: &str = r#"
def env_set(key, value):
    return {"op": "set", "key": key, "value": value}

def env_prepend(key, value, sep = None):
    op = {"op": "prepend", "key": key, "value": value}
    if sep != None:
        op["sep"] = sep
    return op

def env_append(key, value, sep = None):
    op = {"op": "append", "key": key, "value": value}
    if sep != None:
        op["sep"] = sep
    return op

def env_unset(key):
    return {"op": "unset", "key": key}
"#;

/// Standard mock for @vx//stdlib:http.star
pub const MOCK_HTTP_STAR: &str = r#"
def fetch_json_versions(_ctx, _url, _kind):
    """Mock: returns an empty descriptor."""
    return {"kind": _kind, "url": _url}

def fetch_versions_from_api(_ctx, _url, _kind):
    """Mock: returns an empty descriptor."""
    return {"kind": _kind, "url": _url}
"#;

/// Standard mock for @vx//stdlib:github.star
pub const MOCK_GITHUB_STAR: &str = r#"
def make_fetch_versions(_owner, _repo):
    """Mock: returns a fetch_versions function."""
    def _fetch_versions(_ctx):
        return []
    return _fetch_versions

def github_asset_url(owner, repo, tag, asset):
    """Mock: returns a GitHub asset URL."""
    return "https://github.com/" + owner + "/" + repo + "/releases/download/" + tag + "/" + asset

def github_releases(ctx, owner = None, repo = None, include_prereleases = False):
    """Mock: returns an empty list of releases."""
    return []

def releases_to_versions(releases):
    """Mock: converts releases list to version strings."""
    return []
"#;

/// Standard mock for @vx//stdlib:install.star
pub const MOCK_INSTALL_STAR: &str = r#"
def ensure_dependencies(_runtime, check_file = None, lock_file = None, install_dir = None):
    return {"op": "ensure_dependencies", "runtime": _runtime}

def pre_run_ensure_deps(_runtime, trigger_args = None, check_file = None, lock_file = None, install_dir = None):
    return {"op": "pre_run_ensure_deps", "runtime": _runtime}

def dep_def(_runtime, optional = False, reason = ""):
    return {"runtime": _runtime, "optional": optional, "reason": reason}
"#;

/// Standard mock for @vx//stdlib:provider.star
///
/// This mock re-exports ALL symbols that the real provider.star re-exports from
/// its sub-modules (runtime, platform, permissions, layout, system_install,
/// script_install, provider_templates).
pub const MOCK_PROVIDER_STAR: &str = r#"
# --- runtime.star ---
def runtime_def(name, executable = None, aliases = None, description = None,
                priority = 100, auto_installable = True, platform_constraint = None,
                bundled_with = None, system_paths = None, test_commands = None,
                version_pattern = None, **kwargs):
    result = {"name": name, "executable": executable or name}
    if aliases != None:
        result["aliases"] = aliases
    if description != None:
        result["description"] = description
    if platform_constraint != None:
        result["platform_constraint"] = platform_constraint
    if bundled_with != None:
        result["bundled_with"] = bundled_with
    if system_paths != None:
        result["system_paths"] = system_paths
    if test_commands != None:
        result["test_commands"] = test_commands
    if version_pattern != None:
        result["version_pattern"] = version_pattern
    return result

def bundled_runtime_def(name, bundled_with, executable = None, aliases = None,
                        description = None, version_pattern = None, **kwargs):
    result = {"name": name, "executable": executable or name, "bundled_with": bundled_with}
    if aliases != None:
        result["aliases"] = aliases
    if description != None:
        result["description"] = description
    if version_pattern != None:
        result["version_pattern"] = version_pattern
    return result

def dep_def(runtime, optional = False, reason = ""):
    return {"runtime": runtime, "optional": optional, "reason": reason}

# --- platform.star ---
def platform_map(ctx, mapping):
    key = ctx.platform.os + "/" + ctx.platform.arch
    return mapping.get(key)

def platform_select(ctx, windows = None, macos = None, linux = None, default = None):
    os = ctx.platform.os
    if os == "windows" and windows != None:
        return windows
    if os == "macos" and macos != None:
        return macos
    if os == "linux" and linux != None:
        return linux
    return default

# --- permissions.star ---
def github_permissions(extra_hosts = None):
    return []

def system_permissions(**kwargs):
    return []

# --- layout.star ---
def archive_layout(executable, strip_prefix = None):
    def _layout(ctx, version):
        exe = executable + (".exe" if ctx.platform.os == "windows" else "")
        return {"type": "archive", "strip_prefix": strip_prefix or "", "executable_paths": [exe, executable]}
    return _layout

def binary_layout(executable):
    def _layout(ctx, _version):
        exe = executable + (".exe" if ctx.platform.os == "windows" else "")
        return {"type": "binary", "executable_paths": [exe, executable]}
    return _layout

def bin_subdir_layout(executables, strip_prefix = None):
    def _layout(ctx, version):
        return {"type": "archive", "strip_prefix": strip_prefix or "", "executable_paths": executables}
    return _layout

def post_extract_flatten(**kwargs):
    return {"action": "flatten"}

def post_extract_shim(**kwargs):
    return {"action": "shim"}

def post_extract_permissions(paths = None, **kwargs):
    return []

def post_extract_combine(*actions):
    return list(actions)

def pre_run_ensure_deps(runtime, trigger_args = None, check_file = None, lock_file = None, install_dir = None):
    return {"runtime": runtime, "trigger_args": trigger_args, "check_file": check_file}

def fetch_versions_from_api(url, kind):
    return {"url": url, "kind": kind}

def fetch_versions_with_tag_prefix(owner, repo, tag_prefix = "v", prereleases = False):
    return {"owner": owner, "repo": repo, "tag_prefix": tag_prefix}

def bin_subdir_env(subdir = None):
    if subdir:
        return lambda ctx, _version: [
            {"op": "prepend", "key": "PATH", "value": ctx.install_dir + "/" + subdir + "/bin"}
        ]
    else:
        return lambda ctx, _version: [
            {"op": "prepend", "key": "PATH", "value": ctx.install_dir + "/bin"}
        ]

def bin_subdir_execute_path(executable):
    def _get_execute_path(ctx, _version):
        return ctx.install_dir + "/" + executable + ".exe"
    return _get_execute_path

def path_fns(store_name, executable = None):
    exe_name = executable if executable != None else store_name
    def _store_root(ctx):
        return ctx.vx_home + "/store/" + store_name
    def _get_execute_path(ctx, _version):
        return ctx.install_dir + "/" + exe_name + ".exe"
    return {"store_root": _store_root, "get_execute_path": _get_execute_path}

def path_env_fns(extra_env = None):
    def _environment(ctx, _version):
        ops = [{"op": "prepend", "key": "PATH", "value": ctx.install_dir}]
        if extra_env != None:
            ops = ops + extra_env
        return ops
    def _post_install(_ctx, _version):
        return None
    return {"environment": _environment, "post_install": _post_install}

# --- system_install.star ---
def pkg_strategy(manager, package, **kwargs):
    return {"manager": manager, "package": package}

def system_install_strategies(strategies):
    return strategies

def winget_install(package, **kwargs):
    return {"manager": "winget", "package": package}

def choco_install(package, **kwargs):
    return {"manager": "choco", "package": package}

def scoop_install(package, **kwargs):
    return {"manager": "scoop", "package": package}

def brew_install(package, **kwargs):
    return {"manager": "brew", "package": package}

def apt_install(package, **kwargs):
    return {"manager": "apt", "package": package}

def dnf_install(package, **kwargs):
    return {"manager": "dnf", "package": package}

def pacman_install(package, **kwargs):
    return {"manager": "pacman", "package": package}

def snap_install(package, **kwargs):
    return {"manager": "snap", "package": package}

def cross_platform_install(**kwargs):
    return kwargs

def windows_install(**kwargs):
    return kwargs

def multi_platform_install(**kwargs):
    return kwargs

# --- script_install.star ---
def curl_bash_install(url, **kwargs):
    return {"method": "curl_bash", "url": url}

def curl_sh_install(url, **kwargs):
    return {"method": "curl_sh", "url": url}

def irm_iex_install(url, **kwargs):
    return {"method": "irm_iex", "url": url}

def irm_install(url, **kwargs):
    return {"method": "irm", "url": url}

def platform_script_install(**kwargs):
    return kwargs

# --- provider_templates.star ---
def github_rust_provider(owner, repo, executable, asset_pattern = None,
                         strip_prefix = None, **kwargs):
    return {"type": "github_rust", "owner": owner, "repo": repo, "executable": executable}

def github_go_provider(owner, repo, executable, asset_pattern = None,
                       strip_prefix = None, **kwargs):
    return {"type": "github_go", "owner": owner, "repo": repo, "executable": executable}

def github_binary_provider(owner, repo, executable, asset_pattern = None, **kwargs):
    return {"type": "github_binary", "owner": owner, "repo": repo, "executable": executable}

def system_provider(**kwargs):
    return {"type": "system"}

# --- misc ---
def set_permissions(path, mode):
    return {"op": "set_permissions", "path": path, "mode": mode}
"#;

/// Standard mock for @vx//stdlib:system.star (if needed)
pub const MOCK_SYSTEM_STAR: &str = r#"
def system_permissions(**kwargs):
    return []

def post_extract_permissions(paths = None, **kwargs):
    return []
"#;

/// Sets up all standard provider test mocks on the given Assert instance.
///
/// This registers mocks for:
/// - `@vx//stdlib:env.star`
/// - `@vx//stdlib:http.star`
/// - `@vx//stdlib:install.star`
/// - `@vx//stdlib:provider.star`
/// - `@vx//stdlib:system.star`
pub fn setup_provider_test_mocks(a: &mut Assert<'static>) {
    a.module("@vx//stdlib:env.star", MOCK_ENV_STAR);
    a.module("@vx//stdlib:http.star", MOCK_HTTP_STAR);
    a.module("@vx//stdlib:github.star", MOCK_GITHUB_STAR);
    a.module("@vx//stdlib:install.star", MOCK_INSTALL_STAR);
    a.module("@vx//stdlib:provider.star", MOCK_PROVIDER_STAR);
    a.module("@vx//stdlib:system.star", MOCK_SYSTEM_STAR);
}

/// Creates a fully configured Assert instance with all provider test mocks.
///
/// This is a convenience function that creates a new Assert instance,
/// sets the Standard dialect, and registers all standard mocks.
pub fn new_provider_assert() -> Assert<'static> {
    use starlark::syntax::Dialect;

    let mut a = Assert::new();
    a.dialect(&Dialect::Standard);
    setup_provider_test_mocks(&mut a);
    a
}

/// Strips load() statements from starlark source and returns the cleaned source.
///
/// Handles multi-line load() statements by tracking parentheses.
/// Use this when you want to inline provider.star content directly into tests
/// without the load() statements (when using mocks).
pub fn strip_load_statements(source: &str) -> String {
    let mut result = Vec::new();
    let mut in_load = false;
    let mut paren_depth = 0;

    for line in source.lines() {
        let trimmed = line.trim_start();

        if in_load {
            // Count parentheses to find the end of load()
            for ch in line.chars() {
                match ch {
                    '(' => paren_depth += 1,
                    ')' => {
                        paren_depth -= 1;
                        if paren_depth == 0 {
                            in_load = false;
                            break;
                        }
                    }
                    _ => {}
                }
            }
        } else if trimmed.starts_with("load(") {
            // Start of a load statement
            in_load = true;
            paren_depth = 0;
            // Count opening parens
            for ch in line.chars() {
                match ch {
                    '(' => paren_depth += 1,
                    ')' => {
                        paren_depth -= 1;
                        if paren_depth == 0 {
                            in_load = false;
                            break;
                        }
                    }
                    _ => {}
                }
            }
        } else {
            result.push(line);
        }
    }

    result.join("\n")
}

/// Strips load() statements and removes common leading indentation from starlark source.
///
/// Handles multi-line load() statements by tracking parentheses.
/// This is useful for tests that need to evaluate provider.star logic directly
/// without using the module system. The dedented output prevents indentation errors
/// when prepending mock definitions.
pub fn strip_and_dedent_load_statements(source: &str) -> String {
    let lines: Vec<_> = {
        let mut result = Vec::new();
        let mut in_load = false;
        let mut paren_depth = 0;

        for line in source.lines() {
            let trimmed = line.trim_start();

            if in_load {
                // Count parentheses to find the end of load()
                for ch in line.chars() {
                    match ch {
                        '(' => paren_depth += 1,
                        ')' => {
                            paren_depth -= 1;
                            if paren_depth == 0 {
                                in_load = false;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            } else if trimmed.starts_with("load(") {
                // Start of a load statement
                in_load = true;
                paren_depth = 0;
                // Count opening parens
                for ch in line.chars() {
                    match ch {
                        '(' => paren_depth += 1,
                        ')' => {
                            paren_depth -= 1;
                            if paren_depth == 0 {
                                in_load = false;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                result.push(line);
            }
        }
        result
    };

    // Find common leading whitespace (excluding empty lines)
    let non_empty_lines: Vec<_> = lines.iter().filter(|l| !l.trim().is_empty()).collect();
    if non_empty_lines.is_empty() {
        return lines.join("\n");
    }

    let min_indent = non_empty_lines
        .iter()
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    // Remove common leading whitespace
    lines
        .iter()
        .map(|l| {
            if l.trim().is_empty() {
                l.to_string()
            } else {
                l[min_indent.min(l.len())..].to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Creates a test-ready provider.star source by stripping load() statements
/// and prepending mock definitions.
///
/// This is useful for tests that need to evaluate provider.star logic directly
/// without using the module system.
pub fn prepare_provider_source(provider_star: &str) -> String {
    let stripped = strip_and_dedent_load_statements(provider_star);
    format!(
        r#"# Mock definitions (inlined for direct evaluation)

# --- env.star ---
def env_set(key, value):
    return {{"op": "set", "key": key, "value": value}}

def env_prepend(key, value, sep = None):
    op = {{"op": "prepend", "key": key, "value": value}}
    if sep != None:
        op["sep"] = sep
    return op

def env_append(key, value, sep = None):
    op = {{"op": "append", "key": key, "value": value}}
    if sep != None:
        op["sep"] = sep
    return op

def env_unset(key):
    return {{"op": "unset", "key": key}}

# --- http.star ---
def fetch_json_versions(_ctx, _url, _kind):
    return {{"kind": _kind, "url": _url}}

def fetch_versions_from_api(url, kind):
    return {{"url": url, "kind": kind}}

# --- runtime.star ---
def runtime_def(name, executable = None, aliases = None, description = None,
                priority = 100, auto_installable = True, platform_constraint = None,
                bundled_with = None, system_paths = None, test_commands = None,
                version_pattern = None, **kwargs):
    result = {{"name": name, "executable": executable or name}}
    if aliases != None:
        result["aliases"] = aliases
    if description != None:
        result["description"] = description
    if platform_constraint != None:
        result["platform_constraint"] = platform_constraint
    if bundled_with != None:
        result["bundled_with"] = bundled_with
    if system_paths != None:
        result["system_paths"] = system_paths
    if test_commands != None:
        result["test_commands"] = test_commands
    if version_pattern != None:
        result["version_pattern"] = version_pattern
    return result

def bundled_runtime_def(name, bundled_with, executable = None, aliases = None,
                        description = None, version_pattern = None, **kwargs):
    result = {{"name": name, "executable": executable or name, "bundled_with": bundled_with}}
    if aliases != None:
        result["aliases"] = aliases
    if description != None:
        result["description"] = description
    if version_pattern != None:
        result["version_pattern"] = version_pattern
    return result

def dep_def(runtime, optional = False, reason = ""):
    return {{"runtime": runtime, "optional": optional, "reason": reason}}

def ensure_dependencies(_runtime, check_file = None, lock_file = None, install_dir = None):
    return {{"op": "ensure_dependencies", "runtime": _runtime}}

# --- platform.star ---
def platform_map(ctx, mapping):
    key = ctx.platform.os + "/" + ctx.platform.arch
    return mapping.get(key)

def platform_select(ctx, windows = None, macos = None, linux = None, default = None):
    os = ctx.platform.os
    if os == "windows" and windows != None:
        return windows
    if os == "macos" and macos != None:
        return macos
    if os == "linux" and linux != None:
        return linux
    return default

# --- permissions.star ---
def github_permissions(extra_hosts = None):
    return []

def system_permissions(**kwargs):
    return []

# --- layout.star ---
def archive_layout(executable, strip_prefix = None):
    def _layout(ctx, version):
        exe = executable + (".exe" if ctx.platform.os == "windows" else "")
        return {{"type": "archive", "strip_prefix": strip_prefix or "", "executable_paths": [exe, executable]}}
    return _layout

def binary_layout(executable):
    def _layout(ctx, _version):
        exe = executable + (".exe" if ctx.platform.os == "windows" else "")
        return {{"type": "binary", "executable_paths": [exe, executable]}}
    return _layout

def bin_subdir_layout(executables, strip_prefix = None):
    def _layout(ctx, version):
        return {{"type": "archive", "strip_prefix": strip_prefix or "", "executable_paths": executables}}
    return _layout

def post_extract_flatten(**kwargs):
    return {{"action": "flatten"}}

def post_extract_shim(**kwargs):
    return {{"action": "shim"}}

def post_extract_permissions(paths = None, **kwargs):
    return []

def post_extract_combine(*actions):
    return list(actions)

def pre_run_ensure_deps(runtime, trigger_args = None, check_file = None, lock_file = None, install_dir = None):
    return {{"runtime": runtime, "trigger_args": trigger_args, "check_file": check_file}}

def fetch_versions_with_tag_prefix(owner, repo, tag_prefix = "v", prereleases = False):
    return {{"owner": owner, "repo": repo, "tag_prefix": tag_prefix}}

def bin_subdir_env(subdir = None):
    if subdir:
        return lambda ctx, _version: [
            {{"op": "prepend", "key": "PATH", "value": ctx.install_dir + "/" + subdir + "/bin"}}
        ]
    else:
        return lambda ctx, _version: [
            {{"op": "prepend", "key": "PATH", "value": ctx.install_dir + "/bin"}}
        ]

def bin_subdir_execute_path(executable):
    def _get_execute_path(ctx, _version):
        return ctx.install_dir + "/" + executable + ".exe"
    return _get_execute_path

def path_fns(store_name, executable = None):
    exe_name = executable if executable != None else store_name
    def _store_root(ctx):
        return ctx.vx_home + "/store/" + store_name
    def _get_execute_path(ctx, _version):
        return ctx.install_dir + "/" + exe_name + ".exe"
    return {{"store_root": _store_root, "get_execute_path": _get_execute_path}}

def path_env_fns(extra_env = None):
    def _environment(ctx, _version):
        ops = [{{"op": "prepend", "key": "PATH", "value": ctx.install_dir}}]
        if extra_env != None:
            ops = ops + extra_env
        return ops
    def _post_install(_ctx, _version):
        return None
    return {{"environment": _environment, "post_install": _post_install}}

# --- system_install.star ---
def pkg_strategy(manager, package, **kwargs):
    return {{"manager": manager, "package": package}}

def system_install_strategies(strategies):
    return strategies

def winget_install(package, **kwargs):
    return {{"manager": "winget", "package": package}}

def choco_install(package, **kwargs):
    return {{"manager": "choco", "package": package}}

def scoop_install(package, **kwargs):
    return {{"manager": "scoop", "package": package}}

def brew_install(package, **kwargs):
    return {{"manager": "brew", "package": package}}

def apt_install(package, **kwargs):
    return {{"manager": "apt", "package": package}}

def dnf_install(package, **kwargs):
    return {{"manager": "dnf", "package": package}}

def pacman_install(package, **kwargs):
    return {{"manager": "pacman", "package": package}}

def snap_install(package, **kwargs):
    return {{"manager": "snap", "package": package}}

def cross_platform_install(**kwargs):
    return kwargs

def windows_install(**kwargs):
    return kwargs

def multi_platform_install(**kwargs):
    return kwargs

# --- script_install.star ---
def curl_bash_install(url, **kwargs):
    return {{"method": "curl_bash", "url": url}}

def curl_sh_install(url, **kwargs):
    return {{"method": "curl_sh", "url": url}}

def irm_iex_install(url, **kwargs):
    return {{"method": "irm_iex", "url": url}}

def irm_install(url, **kwargs):
    return {{"method": "irm", "url": url}}

def platform_script_install(**kwargs):
    return kwargs

# --- provider_templates.star ---
def github_rust_provider(owner, repo, executable, asset_pattern = None,
                         strip_prefix = None, **kwargs):
    return {{"type": "github_rust", "owner": owner, "repo": repo, "executable": executable}}

def github_go_provider(owner, repo, executable, asset_pattern = None,
                       strip_prefix = None, **kwargs):
    return {{"type": "github_go", "owner": owner, "repo": repo, "executable": executable}}

def github_binary_provider(owner, repo, executable, asset_pattern = None, **kwargs):
    return {{"type": "github_binary", "owner": owner, "repo": repo, "executable": executable}}

def system_provider(**kwargs):
    return {{"type": "system"}}

# --- misc ---
def set_permissions(path, mode):
    return {{"op": "set_permissions", "path": path, "mode": mode}}

def make_fetch_versions(_owner, _repo):
    def _fetch_versions(_ctx):
        return []
    return _fetch_versions

def github_asset_url(owner, repo, tag, asset):
    return "https://github.com/" + owner + "/" + repo + "/releases/download/" + tag + "/" + asset

def github_releases(ctx, owner = None, repo = None, include_prereleases = False):
    return []

def releases_to_versions(releases):
    return []

{}
"#,
        stripped
    )
}
