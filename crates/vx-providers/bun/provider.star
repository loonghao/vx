# provider.star - bun provider
#
# Bun: Incredibly fast JavaScript runtime, bundler, test runner, and package manager
# Inheritance pattern: Level 2 (custom download_url for bun's naming convention)
#   - fetch_versions: inherited from github.star
#   - download_url:   custom (bun-{os}-{arch} naming, no version in asset name)
#
# bun releases: https://github.com/oven-sh/bun/releases
# Asset format: bun-{os}-{arch}.zip

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:http.star", "github_releases")
load("@vx//stdlib:install.star", "create_shim", "ensure_dependencies")

load("@vx//stdlib:env.star", "env_prepend")
# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "bun"
description = "Incredibly fast JavaScript runtime, bundler, test runner, and package manager"
homepage    = "https://bun.sh"
repository  = "https://github.com/oven-sh/bun"
license     = "MIT"
ecosystem   = "nodejs"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "bun",
        "executable":  "bun",
        "description": "Bun JavaScript runtime",
        "aliases":     [],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "^\\d+\\.\\d+\\.\\d+"},
            {"command": "{executable} -e \"console.log('ok')\"", "name": "eval_check", "expected_output": "ok"},
        ],
    },
    {
        "name":         "bunx",
        "executable":   "bun",
        "description":  "Bun package runner",
        "bundled_with": "bun",
        "command_prefix": ["x"],
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — custom (bun uses "bun-v{version}" tag format)
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch bun versions from GitHub releases.

    bun uses non-standard tag format: "bun-v1.2.3" instead of "v1.2.3".
    We strip the "bun-v" prefix to get clean version numbers.
    """
    releases = github_releases(ctx, "oven-sh", "bun", include_prereleases = False)
    # releases is a github_releases descriptor; wrap it with tag_prefix stripping
    return {
        "__type":           "github_versions",
        "source":           releases,
        "tag_key":          "tag_name",
        "strip_v_prefix":   False,
        "tag_prefix":       "bun-v",
        "skip_prereleases": True,
    }

# ---------------------------------------------------------------------------
# download_url — custom
#
# bun asset naming: bun-{os}-{arch}.zip
#   bun-windows-x64.zip
#   bun-darwin-x64.zip / bun-darwin-aarch64.zip
#   bun-linux-x64.zip / bun-linux-aarch64.zip
# ---------------------------------------------------------------------------

def _bun_platform(ctx):
    """Map platform to bun's naming convention."""
    os   = ctx.platform.os
    arch = ctx.platform.arch

    platform_map = {
        "windows/x64":   ("windows", "x64"),
        "macos/x64":     ("darwin",  "x64"),
        "macos/arm64":   ("darwin",  "aarch64"),
        "linux/x64":     ("linux",   "x64"),
        "linux/arm64":   ("linux",   "aarch64"),
    }
    return platform_map.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the bun download URL.

    Args:
        ctx:     Provider context
        version: Version string, e.g. "1.2.3"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _bun_platform(ctx)
    if not platform:
        return None

    bun_os, bun_arch = platform
    asset = "bun-{}-{}.zip".format(bun_os, bun_arch)
    tag = "bun-v{}".format(version)
    return github_asset_url("oven-sh", "bun", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    platform = _bun_platform(ctx)
    os = ctx.platform.os
    exe = "bun.exe" if os == "windows" else "bun"

    if platform:
        bun_os, bun_arch = platform
        strip_prefix = "bun-{}-{}".format(bun_os, bun_arch)
    else:
        strip_prefix = ""

    return {
        "type":             "archive",
        "strip_prefix":     strip_prefix,
        "executable_paths": [exe, "bun"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

# ---------------------------------------------------------------------------
# post_extract — create bunx shim after installation
#
# bun's official distribution does NOT include a standalone `bunx` executable.
# We create a shim script that forwards `bunx <args>` → `bun x <args>`.
# ---------------------------------------------------------------------------

def post_extract(ctx, version, install_dir):
    """Create the bunx shim after bun is extracted.

    bun's zip archive only contains the `bun` executable. We need to create
    a `bunx` shim that wraps `bun x` so that subprocess calls to `bunx` work.

    Args:
        ctx:         Provider context
        version:     Installed version string
        install_dir: Path to the installation directory

    Returns:
        List of post-extract actions
    """
    return [
        create_shim("bunx", "bun", args = ["x"]),
    ]

# ---------------------------------------------------------------------------
# pre_run — ensure node_modules before `bun run`
# ---------------------------------------------------------------------------

def pre_run(ctx, args, executable):
    """Ensure project dependencies are installed before running bun commands.

    For `bun run` commands, checks if node_modules exists and runs
    `bun install` if not. This mirrors the behavior of the Rust runtime's
    `ensure_node_modules_installed()` helper.

    Args:
        ctx:        Provider context
        args:       Command-line arguments passed to bun
        executable: Path to the bun executable

    Returns:
        List of pre-run actions
    """
    # Only trigger for `bun run` commands
    if len(args) > 0 and args[0] == "run":
        return [
            ensure_dependencies(
                "bun",
                check_file  = "package.json",
                lock_file   = "bun.lockb",
                install_dir = "node_modules",
            ),
        ]
    return []


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for bun."""
    return ctx.vx_home + "/store/bun"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx.platform.os
    exe = "bun.exe" if os == "windows" else "bun"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    """Post-install hook (no-op for bun)."""
    return None
