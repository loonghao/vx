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
load("@vx//stdlib:install.star", "ensure_dependencies", "run_command")
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

_PNPM_PLATFORMS = {
    "windows/x64":  ("win",   "x64"),
    "macos/x64":    ("macos", "x64"),
    "macos/arm64":  ("macos", "arm64"),
    "linux/x64":    ("linux", "x64"),
    "linux/arm64":  ("linux", "arm64"),
}

def _pnpm_platform(ctx):
    return _PNPM_PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url — single binary
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _pnpm_platform(ctx)
    if not platform:
        return None
    pnpm_os, pnpm_arch = platform[0], platform[1]
    if ctx.platform.os == "windows":
        asset = "pnpm-{}-{}.exe".format(pnpm_os, pnpm_arch)
    else:
        asset = "pnpm-{}-{}".format(pnpm_os, pnpm_arch)
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
    platform = _pnpm_platform(ctx)
    if not platform:
        return []
    pnpm_os, pnpm_arch = platform[0], platform[1]
    if ctx.platform.os == "windows":
        src = "pnpm-{}-{}.exe".format(pnpm_os, pnpm_arch)
        dst = "pnpm.exe"
    else:
        src = "pnpm-{}-{}".format(pnpm_os, pnpm_arch)
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

# ---------------------------------------------------------------------------
# post_extract — rename platform-specific binary to standard name
#
# pnpm distributes binaries as pnpm-{os}-{arch}[.exe].
# After BinaryHandler extraction, we rename to the standard pnpm[.exe] name.
# ---------------------------------------------------------------------------

def post_extract(ctx, version, install_dir):
    """Rename the downloaded pnpm binary to the standard executable name.

    pnpm releases binaries as e.g. pnpm-linux-x64, pnpm-win-x64.exe.
    The BinaryHandler places them in bin/. We rename to pnpm / pnpm.exe
    so that the executor can find the executable by its standard name.

    Args:
        ctx:         Provider context
        version:     Installed version string
        install_dir: Path to the installation directory

    Returns:
        List of post-extract actions
    """
    platform = _pnpm_platform(ctx)
    if not platform:
        return []

    pnpm_os, pnpm_arch = platform
    os = ctx.platform.os

    if os == "windows":
        source_name = "pnpm-{}-{}.exe".format(pnpm_os, pnpm_arch)
        target_name = "pnpm.exe"
    else:
        source_name = "pnpm-{}-{}".format(pnpm_os, pnpm_arch)
        target_name = "pnpm"

    # Only rename if source differs from target (they always differ for pnpm)
    return [
        run_command(
            "mv" if os != "windows" else "move",
            [ctx.install_dir + "/bin/" + source_name, ctx.install_dir + "/bin/" + target_name],
            on_failure = "warn",
        ),
    ]

# ---------------------------------------------------------------------------
# store_root — vx-managed install directory
# ---------------------------------------------------------------------------

def store_root(ctx, version):
    """Return the vx store root for this pnpm version."""
    return ctx.paths.store_dir + "/pnpm/" + version

# ---------------------------------------------------------------------------
# get_execute_path — resolve pnpm executable
# ---------------------------------------------------------------------------

def get_execute_path(ctx, _version, install_dir):
    """Return the path to the pnpm executable."""
    os  = ctx.platform.os
    exe = "pnpm.exe" if os == "windows" else "pnpm"
    return ctx.install_dir + "/bin/" + exe

# ---------------------------------------------------------------------------
# post_install — set permissions on Unix
# ---------------------------------------------------------------------------

def post_install(ctx, _version):
    """Set execute permissions on Unix."""
    os = ctx.platform.os
    if os == "windows":
        return []
    return [
        {"type": "set_permissions", "path": ctx.install_dir + "/bin/pnpm", "mode": "755"},
    ]

# ---------------------------------------------------------------------------
# pre_run — ensure node_modules before `pnpm run` / `pnpm run-script`
# ---------------------------------------------------------------------------

def pre_run(ctx, args, executable):
    """Ensure project dependencies are installed before running pnpm scripts.

    For `pnpm run` and `pnpm run-script` commands, checks if node_modules
    exists and runs `pnpm install` if not.

    Args:
        ctx:        Provider context
        args:       Command-line arguments passed to pnpm
        executable: Path to the pnpm executable

    Returns:
        List of pre-run actions
    """
    if len(args) > 0 and (args[0] == "run" or args[0] == "run-script"):
        return [
            ensure_dependencies(
                "pnpm",
                check_file  = "package.json",
                install_dir = "node_modules",
            ),
        ]
    return []
