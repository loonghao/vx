# provider.star - pnpm provider
#
# pnpm: Fast, disk space efficient package manager
# Asset: pnpm-{os}-{arch}[.exe] (single binary, no archive)
# Requires Node.js (version-dependent)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "pre_run_ensure_deps")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:install.star", "run_command")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "pnpm"
description = "Fast, disk space efficient package manager"
homepage    = "https://pnpm.io"
repository  = "https://github.com/pnpm/pnpm"
license     = "MIT"
ecosystem   = "nodejs"

# Supported package prefixes for ecosystem:package syntax (RFC 0027)
# Enables `vx pnpm:<package>` for Node.js package execution via pnpm
package_prefixes = ["pnpm"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("pnpm",
        aliases  = ["pnpx"],
        priority = 85,
        version_pattern = "^\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("vx-org", "mirrors", tag_prefix = "pnpm-")

# ---------------------------------------------------------------------------
# Platform helpers
# pnpm asset naming changed in v11:
#   v10 and below: pnpm-{os}-{arch}[.exe]  (single binary)
#   v11 and above: pnpm-{os}-{arch}.{ext}   (archive: tar.gz / zip)
# ---------------------------------------------------------------------------

def _is_v11_plus(version):
    parts = version.split(".")
    major = int(parts[0]) if parts else 0
    return major >= 11

def _pnpm_platform_suffix(ctx, version):
    if _is_v11_plus(version):
        platforms = {
            "windows/x64":  "win32-x64",
            "windows/arm64": "win32-arm64",
            "macos/x64":    "darwin-x64",
            "macos/arm64":  "darwin-arm64",
            "linux/x64":    "linux-x64",
            "linux/arm64":  "linux-arm64",
        }
    else:
        platforms = {
            "windows/x64":  "win-x64",
            "windows/arm64": "win-arm64",
            "macos/x64":    "macos-x64",
            "macos/arm64":  "macos-arm64",
            "linux/x64":    "linux-x64",
            "linux/arm64":  "linux-arm64",
        }
    return platforms.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url — single binary
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform_suffix = _pnpm_platform_suffix(ctx, version)
    if not platform_suffix:
        return None
    if _is_v11_plus(version):
        ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
        asset = "pnpm-{}.{}".format(platform_suffix, ext)
    else:
        if ctx.platform.os == "windows":
            asset = "pnpm-{}.exe".format(platform_suffix)
        else:
            asset = "pnpm-{}".format(platform_suffix)
    return github_asset_url("vx-org", "mirrors", "pnpm-" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — binary
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    platform_suffix = _pnpm_platform_suffix(ctx, version)
    if not platform_suffix:
        return None
    if _is_v11_plus(version):
        return {
            "__type":           "archive",
            "strip_prefix":     "",
            "executable_paths": ["pnpm.exe" if ctx.platform.os == "windows" else "pnpm"],
        }
    if ctx.platform.os == "windows":
        source = "pnpm-{}.exe".format(platform_suffix)
        target = "pnpm.exe"
    else:
        source = "pnpm-{}".format(platform_suffix)
        target = "pnpm"
    return {
        "__type":             "binary_install",
        "source_name":        source,
        "target_name":        target,
        "target_dir":         "bin",
        "target_permissions": "755",
        "executable_paths":   ["bin/" + target],
    }

# ---------------------------------------------------------------------------
# post_extract — rename platform-specific binary to standard name
# ---------------------------------------------------------------------------

def post_extract(ctx, version, _install_dir):
    if _is_v11_plus(version):
        return []
    platform_suffix = _pnpm_platform_suffix(ctx, version)
    if not platform_suffix:
        return []
    if ctx.platform.os == "windows":
        src = "pnpm-{}.exe".format(platform_suffix)
        dst = "pnpm.exe"
    else:
        src = "pnpm-{}".format(platform_suffix)
        dst = "pnpm"
    return [
        run_command(
            "mv" if ctx.platform.os != "windows" else "move",
            [ctx.install_dir + "/bin/" + src, ctx.install_dir + "/bin/" + dst],
            on_failure = "warn",
        ),
    ]

# ---------------------------------------------------------------------------
# pre_run — ensure node_modules before `pnpm run`
# ---------------------------------------------------------------------------

pre_run = pre_run_ensure_deps("pnpm",
    trigger_args = ["run", "run-script"],
    check_file   = "package.json",
    install_dir  = "node_modules",
)

# ---------------------------------------------------------------------------
# deps — version-based Node.js dependency
# ---------------------------------------------------------------------------

def deps(_ctx, version):
    parts = version.split(".")
    major = int(parts[0]) if parts else 0
    if major >= 9:
        return [{"runtime": "node", "version": ">=18"}]
    elif major >= 8:
        return [{"runtime": "node", "version": ">=16"}]
    else:
        return [{"runtime": "node", "version": ">=14"}]

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/pnpm"

def get_execute_path(ctx, version):
    if _is_v11_plus(version):
        return ctx.install_dir + "/pnpm" if ctx.platform.os != "windows" else ctx.install_dir + "/pnpm.exe"
    exe = "pnpm.exe" if ctx.platform.os == "windows" else "pnpm"
    return ctx.install_dir + "/bin/" + exe

def post_install(ctx, version):
    if ctx.platform.os == "windows":
        return []
    if _is_v11_plus(version):
        return [{"type": "set_permissions", "path": ctx.install_dir + "/pnpm", "mode": "755"}]
    return [{"type": "set_permissions", "path": ctx.install_dir + "/bin/pnpm", "mode": "755"}]

def environment(ctx, version):
    if _is_v11_plus(version):
        return [env_prepend("PATH", ctx.install_dir)]
    return [env_prepend("PATH", ctx.install_dir + "/bin")]
