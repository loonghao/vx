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

fetch_versions = make_fetch_versions("pnpm", "pnpm")

# ---------------------------------------------------------------------------
# Platform helpers
# pnpm asset: pnpm-{os}-{arch}[.exe]  (win/macos/linux × x64/arm64)
# ---------------------------------------------------------------------------

# pnpm asset: pnpm-{os}-{arch}[.exe] (all lowercase)
# Actual assets: pnpm-linux-x64, pnpm-macos-arm64, pnpm-win-x64.exe, etc.
_PNPM_PLATFORMS = {
    "windows/x64":  "win-x64",
    "windows/arm64": "win-arm64",
    "macos/x64":    "macos-x64",
    "macos/arm64":  "macos-arm64",
    "linux/x64":    "linux-x64",
    "linux/arm64":  "linux-arm64",
}

def _pnpm_platform_suffix(ctx):
    return _PNPM_PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url — single binary
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform_suffix = _pnpm_platform_suffix(ctx)
    if not platform_suffix:
        return None
    if ctx.platform.os == "windows":
        asset = "pnpm-{}.exe".format(platform_suffix)
    else:
        asset = "pnpm-{}".format(platform_suffix)
    return github_asset_url("pnpm", "pnpm", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — binary
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "pnpm.exe" if ctx.platform.os == "windows" else "pnpm"
    return {
        "type":               "binary",
        "target_name":        exe,
        "target_dir":         "bin",
        "target_permissions": "755",
    }

# ---------------------------------------------------------------------------
# post_extract — rename platform-specific binary to standard name
# ---------------------------------------------------------------------------

def post_extract(ctx, _version, _install_dir):
    platform_suffix = _pnpm_platform_suffix(ctx)
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

def get_execute_path(ctx, _version):
    exe = "pnpm.exe" if ctx.platform.os == "windows" else "pnpm"
    return ctx.install_dir + "/bin/" + exe

def post_install(ctx, _version):
    if ctx.platform.os == "windows":
        return []
    return [{"type": "set_permissions", "path": ctx.install_dir + "/bin/pnpm", "mode": "755"}]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]
