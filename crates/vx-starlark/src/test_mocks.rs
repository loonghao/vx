//! Standard mocks for Starlark provider tests
//!
//! This module provides pre-built mock implementations for `@vx//stdlib/*`
//! modules used in provider.star files. Use these to set up test environments
//! without duplicating mock code across provider tests.
//!
//! # Example
//!
//! ```rust,ignore
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
def make_fetch_versions(_owner, _repo, include_prereleases = False, **kwargs):
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

def dep_def(_runtime, version = "*", optional = False, reason = None,
            recommended = None, provided_by = None):
    result = {"runtime": _runtime, "version": version, "optional": optional}
    if reason != None:
        result["reason"] = reason
    if recommended != None:
        result["recommended"] = recommended
    if provided_by != None:
        result["provided_by"] = provided_by
    return result

def create_shim(name, target_executable, args = None, shim_dir = None):
    result = {"__type": "create_shim", "name": name, "target": target_executable}
    if args != None:
        result["args"] = args
    if shim_dir != None:
        result["shim_dir"] = shim_dir
    return result

def set_permissions(path, mode = "755"):
    return {"__type": "set_permissions", "path": path, "mode": mode}

def run_command(executable, args, working_dir = None, env = None, on_failure = "warn"):
    result = {
        "__type": "run_command",
        "executable": executable,
        "args": args,
        "on_failure": on_failure,
    }
    if working_dir != None:
        result["working_dir"] = working_dir
    if env != None:
        result["env"] = env
    return result

def flatten_dir(pattern = None, keep_subdirs = None):
    result = {"__type": "flatten_dir"}
    if pattern != None:
        result["pattern"] = pattern
    if keep_subdirs != None:
        result["keep_subdirs"] = keep_subdirs
    return result
"#;

/// Standard mock for @vx//stdlib:provider.star
///
/// This mock re-exports ALL symbols that the real provider.star re-exports from
/// its sub-modules (runtime, platform, permissions, layout, system_install,
/// script_install, provider_templates).
pub const MOCK_PROVIDER_STAR: &str = r#"
# --- runtime.star ---
def runtime_def(name, executable = None, aliases = None, description = None,
                priority = 100, auto_installable = None, platform_constraint = None,
                bundled_with = None, system_paths = None, test_commands = None,
                version_pattern = None, **kwargs):
    result = {"name": name, "executable": executable or name}
    if aliases != None:
        result["aliases"] = aliases
    if description != None:
        result["description"] = description
    if auto_installable != None:
        result["auto_installable"] = auto_installable
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
                        description = None, version_pattern = None,
                        auto_installable = None, platform_constraint = None, **kwargs):
    result = {"name": name, "executable": executable or name, "bundled_with": bundled_with}
    if aliases != None:
        result["aliases"] = aliases
    if description != None:
        result["description"] = description
    if version_pattern != None:
        result["version_pattern"] = version_pattern
    if auto_installable != None:
        result["auto_installable"] = auto_installable
    if platform_constraint != None:
        result["platform_constraint"] = platform_constraint
    return result

def dep_def(runtime, version = "*", optional = False, reason = None,
            recommended = None, provided_by = None):
    result = {"runtime": runtime, "version": version, "optional": optional}
    if reason != None:
        result["reason"] = reason
    if recommended != None:
        result["recommended"] = recommended
    if provided_by != None:
        result["provided_by"] = provided_by
    return result

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
def github_permissions(extra_hosts = None, exec_cmds = None, **kwargs):
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

def post_extract_flatten(pattern = None):
    def _post_extract(_ctx, _version, _install_dir):
        result = {"__type": "flatten_dir"}
        if pattern != None:
            result["pattern"] = pattern
        return [result]
    return _post_extract

def post_extract_shim(shim_name = None, target_executable = None, args = None, **kwargs):
    def _post_extract(_ctx, _version, _install_dir):
        result = {"__type": "create_shim"}
        if shim_name != None:
            result["name"] = shim_name
        if target_executable != None:
            result["target"] = target_executable
        if args != None:
            result["args"] = args
        return [result]
    return _post_extract

def post_extract_permissions(paths = None, mode = "755", unix_only = True, **kwargs):
    def _post_extract(ctx, _version, _install_dir):
        if unix_only and ctx.platform.os == "windows":
            return []
        return [{"__type": "set_permissions", "path": p, "mode": mode} for p in (paths or [])]
    return _post_extract

def post_extract_combine(hooks):
    def _post_extract(ctx, version, install_dir):
        result = []
        for hook in hooks:
            result = result + hook(ctx, version, install_dir)
        return result
    return _post_extract

def pre_run_ensure_deps(runtime, trigger_args = None, check_file = None, lock_file = None, install_dir = None):
    def _pre_run(_ctx, args, _executable):
        if trigger_args != None:
            if len(args) == 0 or args[0] not in trigger_args:
                return []
        result = {"__type": "ensure_dependencies", "package_manager": runtime, "check_file": check_file, "install_dir": install_dir}
        if lock_file != None:
            result["lock_file"] = lock_file
        return [result]
    return _pre_run


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
    result = {"manager": manager, "package": package}
    for key in kwargs:
        result[key] = kwargs[key]
    return result

def system_install_strategies(strategies):
    return strategies

def winget_install(package, **kwargs):
    return pkg_strategy("winget", package, **kwargs)

def choco_install(package, **kwargs):
    return pkg_strategy("choco", package, **kwargs)

def scoop_install(package, **kwargs):
    return pkg_strategy("scoop", package, **kwargs)

def brew_install(package, **kwargs):
    return pkg_strategy("brew", package, **kwargs)

def apt_install(package, **kwargs):
    return pkg_strategy("apt", package, **kwargs)

def dnf_install(package, **kwargs):
    return pkg_strategy("dnf", package, **kwargs)

def pacman_install(package, **kwargs):
    return pkg_strategy("pacman", package, **kwargs)

def snap_install(package, **kwargs):
    return pkg_strategy("snap", package, **kwargs)

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
def _mock_exe_suffix(ctx):
    return ".exe" if ctx.platform.os == "windows" else ""

def _mock_archive_ext(ctx):
    return "zip" if ctx.platform.os == "windows" else "tar.gz"

def _mock_rust_triple(ctx, linux_libc = "musl"):
    return {
        "windows/x64": "x86_64-pc-windows-msvc",
        "windows/arm64": "aarch64-pc-windows-msvc",
        "macos/x64": "x86_64-apple-darwin",
        "macos/arm64": "aarch64-apple-darwin",
        "linux/x64": "x86_64-unknown-linux-" + linux_libc,
        "linux/arm64": "aarch64-unknown-linux-" + linux_libc,
        "linux/armv7": "armv7-unknown-linux-gnueabihf",
    }.get(ctx.platform.os + "/" + ctx.platform.arch)

def _mock_go_os_arch(ctx):
    return {
        "windows/x64": ("windows", "amd64"),
        "windows/arm64": ("windows", "arm64"),
        "macos/x64": ("darwin", "amd64"),
        "macos/arm64": ("darwin", "arm64"),
        "linux/x64": ("linux", "amd64"),
        "linux/arm64": ("linux", "arm64"),
        "linux/armv7": ("linux", "armv7"),
    }.get(ctx.platform.os + "/" + ctx.platform.arch)

def _mock_expand_asset(asset, ctx, version, triple = None, go_os = None, go_arch = None):
    result = asset
    for key, value in [
        ("{version}", version),
        ("{vversion}", "v" + version),
        ("{triple}", triple or ""),
        ("{os}", go_os or ""),
        ("{arch}", go_arch or ""),
        ("{go_os}", go_os or ""),
        ("{go_arch}", go_arch or ""),
        ("{ext}", _mock_archive_ext(ctx)),
        ("{exe}", _mock_exe_suffix(ctx)),
    ]:
        result = result.replace(key, value)
    return result

def _mock_std_provider_fns(store_name, exe_name, path_env = True, extra_env = None):
    def _store_root(ctx):
        return ctx.vx_home + "/store/" + store_name

    def _get_execute_path(ctx, _version):
        return ctx.install_dir + "/" + exe_name + _mock_exe_suffix(ctx)

    def _post_install(_ctx, _version):
        return None

    def _environment(ctx, _version):
        ops = [{"op": "prepend", "key": "PATH", "value": ctx.install_dir}] if path_env else []
        if extra_env != None:
            ops = ops + extra_env
        return ops

    def _deps(_ctx, _version):
        return []

    return {
        "store_root": _store_root,
        "get_execute_path": _get_execute_path,
        "post_install": _post_install,
        "environment": _environment,
        "deps": _deps,
    }

def github_rust_provider(owner, repo, asset = None, executable = None,
                         store = None, tag_prefix = "v", linux_libc = "musl",
                         prereleases = False, strip_prefix = None, path_env = True,
                         extra_env = None, asset_pattern = None, **kwargs):
    exe_name = executable if executable != None else repo
    store_name = store if store != None else repo
    asset_tmpl = asset if asset != None else asset_pattern
    if asset_tmpl == None:
        asset_tmpl = repo + "-{version}-{triple}.{ext}"

    def _fetch_versions(_ctx):
        return []

    def _download_url(ctx, version):
        triple = _mock_rust_triple(ctx, linux_libc)
        if triple == None:
            return None
        fname = _mock_expand_asset(asset_tmpl, ctx, version, triple = triple)
        return "https://github.com/{}/{}/releases/download/{}/{}".format(owner, repo, tag_prefix + version, fname)

    def _install_layout(ctx, version):
        exe = exe_name + _mock_exe_suffix(ctx)
        strip = ""
        if strip_prefix != None:
            triple = _mock_rust_triple(ctx, linux_libc) or ""
            strip = _mock_expand_asset(strip_prefix, ctx, version, triple = triple)
        return {
            "type": "archive",
            "__type": "archive",
            "strip_prefix": strip,
            "executable_paths": [exe, exe_name],
        }

    fns = _mock_std_provider_fns(store_name, exe_name, path_env, extra_env)
    fns["fetch_versions"] = _fetch_versions
    fns["download_url"] = _download_url
    fns["install_layout"] = _install_layout
    return fns

def github_go_provider(owner, repo, asset = None, executable = None,
                       store = None, tag_prefix = "v", prereleases = False,
                       strip_prefix = None, path_env = True, extra_env = None,
                       asset_pattern = None, **kwargs):
    exe_name = executable if executable != None else repo
    store_name = store if store != None else repo
    asset_tmpl = asset if asset != None else asset_pattern
    if asset_tmpl == None:
        asset_tmpl = repo + "_{version}_{os}_{arch}.{ext}"

    def _fetch_versions(_ctx):
        return []

    def _download_url(ctx, version):
        go_target = _mock_go_os_arch(ctx)
        if go_target == None:
            return None
        go_os, go_arch = go_target[0], go_target[1]
        fname = _mock_expand_asset(asset_tmpl, ctx, version, go_os = go_os, go_arch = go_arch)
        return "https://github.com/{}/{}/releases/download/{}/{}".format(owner, repo, tag_prefix + version, fname)

    def _install_layout(ctx, version):
        exe = exe_name + _mock_exe_suffix(ctx)
        strip = ""
        if strip_prefix != None:
            go_target = _mock_go_os_arch(ctx)
            go_os = go_target[0] if go_target != None else ""
            go_arch = go_target[1] if go_target != None else ""
            strip = _mock_expand_asset(strip_prefix, ctx, version, go_os = go_os, go_arch = go_arch)
        return {
            "type": "archive",
            "__type": "archive",
            "strip_prefix": strip,
            "executable_paths": [exe, exe_name],
        }

    fns = _mock_std_provider_fns(store_name, exe_name, path_env, extra_env)
    fns["fetch_versions"] = _fetch_versions
    fns["download_url"] = _download_url
    fns["install_layout"] = _install_layout
    return fns

def github_binary_provider(owner, repo, asset = None, executable = None,
                           store = None, tag_prefix = "v", prereleases = False,
                           path_env = True, extra_env = None, asset_pattern = None,
                           **kwargs):
    exe_name = executable if executable != None else repo
    store_name = store if store != None else repo
    asset_tmpl = asset if asset != None else asset_pattern
    if asset_tmpl == None:
        asset_tmpl = repo + "-{version}-{os}-{arch}{exe}"

    def _fetch_versions(_ctx):
        return []

    def _download_url(ctx, version):
        go_target = _mock_go_os_arch(ctx)
        go_os = go_target[0] if go_target != None else ""
        go_arch = go_target[1] if go_target != None else ""
        triple = _mock_rust_triple(ctx)
        fname = _mock_expand_asset(asset_tmpl, ctx, version, triple = triple, go_os = go_os, go_arch = go_arch)
        return "https://github.com/{}/{}/releases/download/{}/{}".format(owner, repo, tag_prefix + version, fname)

    def _install_layout(ctx, _version):
        exe = exe_name + _mock_exe_suffix(ctx)
        return {
            "type": "binary",
            "__type": "binary",
            "target_name": exe,
            "target_dir": "bin",
            "executable_paths": ["bin/" + exe, exe, exe_name],
        }

    fns = _mock_std_provider_fns(store_name, exe_name, path_env, extra_env)
    fns["fetch_versions"] = _fetch_versions
    fns["download_url"] = _download_url
    fns["install_layout"] = _install_layout
    return fns

def system_provider(store_name, executable = None, path_env = True, extra_env = None, **kwargs):
    exe_name = executable if executable != None else store_name

    def _fetch_versions(_ctx):
        return []

    def _download_url(_ctx, _version):
        return None

    def _install_layout(_ctx, _version):
        return None

    fns = _mock_std_provider_fns(store_name, exe_name, path_env, extra_env)
    fns["fetch_versions"] = _fetch_versions
    fns["download_url"] = _download_url
    fns["install_layout"] = _install_layout
    return fns

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

/// Standard mock for @vx//stdlib:layout.star
pub const MOCK_LAYOUT_STAR: &str = r#"
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

def post_extract_flatten(pattern = None):
    def _post_extract(_ctx, _version, _install_dir):
        result = {"__type": "flatten_dir"}
        if pattern != None:
            result["pattern"] = pattern
        return [result]
    return _post_extract

def post_extract_shim(shim_name = None, target_executable = None, args = None, **kwargs):
    def _post_extract(_ctx, _version, _install_dir):
        result = {"__type": "create_shim"}
        if shim_name != None:
            result["name"] = shim_name
        if target_executable != None:
            result["target"] = target_executable
        if args != None:
            result["args"] = args
        return [result]
    return _post_extract

def post_extract_permissions(paths = None, mode = "755", unix_only = True, **kwargs):
    def _post_extract(ctx, _version, _install_dir):
        if unix_only and ctx.platform.os == "windows":
            return []
        return [{"__type": "set_permissions", "path": p, "mode": mode} for p in (paths or [])]
    return _post_extract

def post_extract_combine(hooks):
    def _post_extract(ctx, version, install_dir):
        result = []
        for hook in hooks:
            result = result + hook(ctx, version, install_dir)
        return result
    return _post_extract

def pre_run_ensure_deps(runtime, trigger_args = None, check_file = None, lock_file = None, install_dir = None):
    def _pre_run(_ctx, args, _executable):
        if trigger_args != None:
            if len(args) == 0 or args[0] not in trigger_args:
                return []
        result = {"__type": "ensure_dependencies", "package_manager": runtime, "check_file": check_file, "install_dir": install_dir}
        if lock_file != None:
            result["lock_file"] = lock_file
        return [result]
    return _pre_run

def fetch_versions_with_tag_prefix(owner, repo, tag_prefix = "v", prereleases = False):
    """Mock: returns a version descriptor with the given tag prefix."""
    return {"__type": "github_versions", "owner": owner, "repo": repo, "tag_prefix": tag_prefix}
"#;

/// Standard mock for @vx//stdlib:platform.star
pub const MOCK_PLATFORM_STAR: &str = r#"
def exe_suffix(ctx):
    return ".exe" if ctx.platform.os == "windows" else ""

def archive_ext(ctx):
    return "zip" if ctx.platform.os == "windows" else "tar.gz"

def rust_triple(ctx, linux_libc = "musl"):
    key = ctx.platform.os + "/" + ctx.platform.arch
    mapping = {
        "windows/x64": "x86_64-pc-windows-msvc",
        "windows/arm64": "aarch64-pc-windows-msvc",
        "macos/x64": "x86_64-apple-darwin",
        "macos/arm64": "aarch64-apple-darwin",
        "linux/x64": "x86_64-unknown-linux-" + linux_libc,
        "linux/arm64": "aarch64-unknown-linux-" + linux_libc,
    }
    return mapping.get(key)

def go_os_arch(ctx):
    key = ctx.platform.os + "/" + ctx.platform.arch
    mapping = {
        "windows/x64": ("windows", "amd64"),
        "windows/arm64": ("windows", "arm64"),
        "macos/x64": ("darwin", "amd64"),
        "macos/arm64": ("darwin", "arm64"),
        "linux/x64": ("linux", "amd64"),
        "linux/arm64": ("linux", "arm64"),
    }
    return mapping.get(key)

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
"#;

/// Sets up all standard provider test mocks on the given Assert instance.
///
/// This registers mocks for:
/// - `@vx//stdlib:env.star`
/// - `@vx//stdlib:http.star`
/// - `@vx//stdlib:github.star`
/// - `@vx//stdlib:install.star`
/// - `@vx//stdlib:layout.star`
/// - `@vx//stdlib:platform.star`
/// - `@vx//stdlib:provider.star`
/// - `@vx//stdlib:system.star`
pub fn setup_provider_test_mocks(a: &mut Assert<'static>) {
    a.module("@vx//stdlib:env.star", MOCK_ENV_STAR);
    a.module("@vx//stdlib:http.star", MOCK_HTTP_STAR);
    a.module("@vx//stdlib:github.star", MOCK_GITHUB_STAR);
    a.module("@vx//stdlib:install.star", MOCK_INSTALL_STAR);
    a.module("@vx//stdlib:layout.star", MOCK_LAYOUT_STAR);
    a.module("@vx//stdlib:platform.star", MOCK_PLATFORM_STAR);
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
                priority = 100, auto_installable = None, platform_constraint = None,
                bundled_with = None, system_paths = None, test_commands = None,
                version_pattern = None, **kwargs):
    result = {{"name": name, "executable": executable or name}}
    if aliases != None:
        result["aliases"] = aliases
    if description != None:
        result["description"] = description
    if auto_installable != None:
        result["auto_installable"] = auto_installable
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
                        description = None, version_pattern = None,
                        auto_installable = None, platform_constraint = None, **kwargs):
    result = {{"name": name, "executable": executable or name, "bundled_with": bundled_with}}
    if aliases != None:
        result["aliases"] = aliases
    if description != None:
        result["description"] = description
    if version_pattern != None:
        result["version_pattern"] = version_pattern
    if auto_installable != None:
        result["auto_installable"] = auto_installable
    if platform_constraint != None:
        result["platform_constraint"] = platform_constraint
    return result

def dep_def(runtime, version = "*", optional = False, reason = None,
            recommended = None, provided_by = None):
    result = {{"runtime": runtime, "version": version, "optional": optional}}
    if reason != None:
        result["reason"] = reason
    if recommended != None:
        result["recommended"] = recommended
    if provided_by != None:
        result["provided_by"] = provided_by
    return result

def ensure_dependencies(_runtime, check_file = None, lock_file = None, install_dir = None):
    return {{"op": "ensure_dependencies", "runtime": _runtime}}

def create_shim(name, target_executable, args = None, shim_dir = None):
    result = {{"__type": "create_shim", "name": name, "target": target_executable}}
    if args != None:
        result["args"] = args
    if shim_dir != None:
        result["shim_dir"] = shim_dir
    return result

def set_permissions(path, mode = "755"):
    return {{"__type": "set_permissions", "path": path, "mode": mode}}

def run_command(executable, args, working_dir = None, env = None, on_failure = "warn"):
    result = {{
        "__type": "run_command",
        "executable": executable,
        "args": args,
        "on_failure": on_failure,
    }}
    if working_dir != None:
        result["working_dir"] = working_dir
    if env != None:
        result["env"] = env
    return result

def flatten_dir(pattern = None, keep_subdirs = None):
    result = {{"__type": "flatten_dir"}}
    if pattern != None:
        result["pattern"] = pattern
    if keep_subdirs != None:
        result["keep_subdirs"] = keep_subdirs
    return result

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

def rust_triple(ctx, linux_libc = "musl"):
    """Mock: returns the Rust target triple for the current platform."""
    return {{
        "windows/x64":   "x86_64-pc-windows-msvc",
        "windows/arm64": "aarch64-pc-windows-msvc",
        "macos/x64":     "x86_64-apple-darwin",
        "macos/arm64":   "aarch64-apple-darwin",
        "linux/x64":     "x86_64-unknown-linux-" + linux_libc,
        "linux/arm64":   "aarch64-unknown-linux-" + linux_libc,
        "linux/armv7":   "armv7-unknown-linux-gnueabihf",
    }}.get(ctx.platform.os + "/" + ctx.platform.arch)

# --- permissions.star ---
def github_permissions(extra_hosts = None, exec_cmds = None, **kwargs):
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

def post_extract_shim(shim_name = None, target_executable = None, args = None, shim_dir = None, **kwargs):
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
    result = {{"manager": manager, "package": package}}
    for key in kwargs:
        result[key] = kwargs[key]
    return result

def system_install_strategies(strategies):
    return strategies

def winget_install(package, **kwargs):
    return pkg_strategy("winget", package, **kwargs)

def choco_install(package, **kwargs):
    return pkg_strategy("choco", package, **kwargs)

def scoop_install(package, **kwargs):
    return pkg_strategy("scoop", package, **kwargs)

def brew_install(package, **kwargs):
    return pkg_strategy("brew", package, **kwargs)

def apt_install(package, **kwargs):
    return pkg_strategy("apt", package, **kwargs)

def dnf_install(package, **kwargs):
    return pkg_strategy("dnf", package, **kwargs)

def pacman_install(package, **kwargs):
    return pkg_strategy("pacman", package, **kwargs)

def snap_install(package, **kwargs):
    return pkg_strategy("snap", package, **kwargs)

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
def _mock_exe_suffix(ctx):
    return ".exe" if ctx.platform.os == "windows" else ""

def _mock_archive_ext(ctx):
    return "zip" if ctx.platform.os == "windows" else "tar.gz"

def _mock_rust_triple(ctx, linux_libc = "musl"):
    return {{
        "windows/x64": "x86_64-pc-windows-msvc",
        "windows/arm64": "aarch64-pc-windows-msvc",
        "macos/x64": "x86_64-apple-darwin",
        "macos/arm64": "aarch64-apple-darwin",
        "linux/x64": "x86_64-unknown-linux-" + linux_libc,
        "linux/arm64": "aarch64-unknown-linux-" + linux_libc,
        "linux/armv7": "armv7-unknown-linux-gnueabihf",
    }}.get(ctx.platform.os + "/" + ctx.platform.arch)

def _mock_go_os_arch(ctx):
    return {{
        "windows/x64": ("windows", "amd64"),
        "windows/arm64": ("windows", "arm64"),
        "macos/x64": ("darwin", "amd64"),
        "macos/arm64": ("darwin", "arm64"),
        "linux/x64": ("linux", "amd64"),
        "linux/arm64": ("linux", "arm64"),
        "linux/armv7": ("linux", "armv7"),
    }}.get(ctx.platform.os + "/" + ctx.platform.arch)

def _mock_expand_asset(asset, ctx, version, triple = None, go_os = None, go_arch = None):
    result = asset
    for key, value in [
        ("{{version}}", version),
        ("{{vversion}}", "v" + version),
        ("{{triple}}", triple or ""),
        ("{{os}}", go_os or ""),
        ("{{arch}}", go_arch or ""),
        ("{{go_os}}", go_os or ""),
        ("{{go_arch}}", go_arch or ""),
        ("{{ext}}", _mock_archive_ext(ctx)),
        ("{{exe}}", _mock_exe_suffix(ctx)),
    ]:
        result = result.replace(key, value)
    return result

def _mock_std_provider_fns(store_name, exe_name, path_env = True, extra_env = None):
    def _store_root(ctx):
        return ctx.vx_home + "/store/" + store_name

    def _get_execute_path(ctx, _version):
        return ctx.install_dir + "/" + exe_name + _mock_exe_suffix(ctx)

    def _post_install(_ctx, _version):
        return None

    def _environment(ctx, _version):
        ops = [{{"op": "prepend", "key": "PATH", "value": ctx.install_dir}}] if path_env else []
        if extra_env != None:
            ops = ops + extra_env
        return ops

    def _deps(_ctx, _version):
        return []

    return {{
        "store_root": _store_root,
        "get_execute_path": _get_execute_path,
        "post_install": _post_install,
        "environment": _environment,
        "deps": _deps,
    }}

def github_rust_provider(owner, repo, asset = None, executable = None,
                         store = None, tag_prefix = "v", linux_libc = "musl",
                         prereleases = False, strip_prefix = None, path_env = True,
                         extra_env = None, asset_pattern = None, **kwargs):
    exe_name = executable if executable != None else repo
    store_name = store if store != None else repo
    asset_tmpl = asset if asset != None else asset_pattern
    if asset_tmpl == None:
        asset_tmpl = repo + "-{{version}}-{{triple}}.{{ext}}"

    def _fetch_versions(_ctx):
        return []

    def _download_url(ctx, version):
        triple = _mock_rust_triple(ctx, linux_libc)
        if triple == None:
            return None
        fname = _mock_expand_asset(asset_tmpl, ctx, version, triple = triple)
        return "https://github.com/{{}}/{{}}/releases/download/{{}}/{{}}".format(owner, repo, tag_prefix + version, fname)

    def _install_layout(ctx, version):
        exe = exe_name + _mock_exe_suffix(ctx)
        strip = ""
        if strip_prefix != None:
            triple = _mock_rust_triple(ctx, linux_libc) or ""
            strip = _mock_expand_asset(strip_prefix, ctx, version, triple = triple)
        return {{
            "type": "archive",
            "__type": "archive",
            "strip_prefix": strip,
            "executable_paths": [exe, exe_name],
        }}

    fns = _mock_std_provider_fns(store_name, exe_name, path_env, extra_env)
    fns["fetch_versions"] = _fetch_versions
    fns["download_url"] = _download_url
    fns["install_layout"] = _install_layout
    return fns

def github_go_provider(owner, repo, asset = None, executable = None,
                       store = None, tag_prefix = "v", prereleases = False,
                       strip_prefix = None, path_env = True, extra_env = None,
                       asset_pattern = None, **kwargs):
    exe_name = executable if executable != None else repo
    store_name = store if store != None else repo
    asset_tmpl = asset if asset != None else asset_pattern
    if asset_tmpl == None:
        asset_tmpl = repo + "_{{version}}_{{os}}_{{arch}}.{{ext}}"

    def _fetch_versions(_ctx):
        return []

    def _download_url(ctx, version):
        go_target = _mock_go_os_arch(ctx)
        if go_target == None:
            return None
        go_os, go_arch = go_target[0], go_target[1]
        fname = _mock_expand_asset(asset_tmpl, ctx, version, go_os = go_os, go_arch = go_arch)
        return "https://github.com/{{}}/{{}}/releases/download/{{}}/{{}}".format(owner, repo, tag_prefix + version, fname)

    def _install_layout(ctx, version):
        exe = exe_name + _mock_exe_suffix(ctx)
        strip = ""
        if strip_prefix != None:
            go_target = _mock_go_os_arch(ctx)
            go_os = go_target[0] if go_target != None else ""
            go_arch = go_target[1] if go_target != None else ""
            strip = _mock_expand_asset(strip_prefix, ctx, version, go_os = go_os, go_arch = go_arch)
        return {{
            "type": "archive",
            "__type": "archive",
            "strip_prefix": strip,
            "executable_paths": [exe, exe_name],
        }}

    fns = _mock_std_provider_fns(store_name, exe_name, path_env, extra_env)
    fns["fetch_versions"] = _fetch_versions
    fns["download_url"] = _download_url
    fns["install_layout"] = _install_layout
    return fns

def github_binary_provider(owner, repo, asset = None, executable = None,
                           store = None, tag_prefix = "v", prereleases = False,
                           path_env = True, extra_env = None, asset_pattern = None,
                           **kwargs):
    exe_name = executable if executable != None else repo
    store_name = store if store != None else repo
    asset_tmpl = asset if asset != None else asset_pattern
    if asset_tmpl == None:
        asset_tmpl = repo + "-{{version}}-{{os}}-{{arch}}{{exe}}"

    def _fetch_versions(_ctx):
        return []

    def _download_url(ctx, version):
        go_target = _mock_go_os_arch(ctx)
        go_os = go_target[0] if go_target != None else ""
        go_arch = go_target[1] if go_target != None else ""
        triple = _mock_rust_triple(ctx)
        fname = _mock_expand_asset(asset_tmpl, ctx, version, triple = triple, go_os = go_os, go_arch = go_arch)
        return "https://github.com/{{}}/{{}}/releases/download/{{}}/{{}}".format(owner, repo, tag_prefix + version, fname)

    def _install_layout(ctx, _version):
        exe = exe_name + _mock_exe_suffix(ctx)
        return {{
            "type": "binary",
            "__type": "binary",
            "target_name": exe,
            "target_dir": "bin",
            "executable_paths": ["bin/" + exe, exe, exe_name],
        }}

    fns = _mock_std_provider_fns(store_name, exe_name, path_env, extra_env)
    fns["fetch_versions"] = _fetch_versions
    fns["download_url"] = _download_url
    fns["install_layout"] = _install_layout
    return fns

def system_provider(store_name, executable = None, path_env = True, extra_env = None, **kwargs):
    exe_name = executable if executable != None else store_name

    def _fetch_versions(_ctx):
        return []

    def _download_url(_ctx, _version):
        return None

    def _install_layout(_ctx, _version):
        return None

    fns = _mock_std_provider_fns(store_name, exe_name, path_env, extra_env)
    fns["fetch_versions"] = _fetch_versions
    fns["download_url"] = _download_url
    fns["install_layout"] = _install_layout
    return fns

# --- misc ---
def set_permissions(path, mode):
    return {{"op": "set_permissions", "path": path, "mode": mode}}

def make_fetch_versions(_owner, _repo, include_prereleases = False, **kwargs):
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
