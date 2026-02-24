# provider.star - rcedit provider (Windows-only)
#
# Reuse pattern: Level 2 (partial override)
#   - fetch_versions: fully inherited from github.star
#   - download_url:   overridden — rcedit is Windows-only, direct binary download
#
# Key characteristics:
#   - Windows-only tool (returns None on non-Windows platforms)
#   - Direct binary download (NOT an archive) — type = "binary"
#   - Asset naming: "rcedit-{arch}.exe"  (x64 / arm64 / x86)
#   - URL: https://github.com/electron/rcedit/releases/download/v{version}/rcedit-{arch}.exe
#   - post_extract renames "rcedit-x64.exe" → "bin/rcedit.exe"
#
# Equivalent Rust replaced:
#   - RceditUrlBuilder::download_url()  → download_url() below
#   - RceditRuntime::post_extract()     → install_layout() rename hint

load("@vx//stdlib:provider.star",
     "github_binary_provider", "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "rcedit"
description = "rcedit - Command-line tool to edit resources of Windows executables"
homepage    = "https://github.com/electron/rcedit"
repository  = "https://github.com/electron/rcedit"
license     = "MIT"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Platform constraint — Windows only
# ---------------------------------------------------------------------------

platforms = {
    "os": ["windows"],
}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("rcedit",
        description   = "Command-line tool to edit resources of Windows executables",
        version_cmd   = "{executable} --help",
        test_commands = [{"command": "{executable} --help", "name": "help_check",
                          "expect_success": True}]),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(
    extra_hosts = ["objects.githubusercontent.com"],
)

# ---------------------------------------------------------------------------
# Provider template — binary download, Windows-only
# ---------------------------------------------------------------------------

_p = github_binary_provider(
    "electron", "rcedit",
    asset      = "rcedit-x64.exe",   # overridden below
    executable = "rcedit",
)

store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]
fetch_versions   = _p["fetch_versions"]

# ---------------------------------------------------------------------------
# download_url — Windows-only, arch-specific binary
#
# Asset naming: rcedit-{arch}.exe  (x64 / arm64 / x86)
# ---------------------------------------------------------------------------

_ARCH_MAP = {
    "x64":   "x64",
    "arm64": "arm64",
    "x86":   "x86",
}

def download_url(ctx, version):
    if ctx.platform.os != "windows":
        return None
    arch = _ARCH_MAP.get(ctx.platform.arch)
    if not arch:
        return None
    asset = "rcedit-{}.exe".format(arch)
    return github_asset_url("electron", "rcedit", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — binary, rename rcedit-{arch}.exe → rcedit.exe
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    arch = _ARCH_MAP.get(ctx.platform.arch, "x64")
    return {
        "type":            "binary",
        "source_name":     "rcedit-{}.exe".format(arch),
        "executable_name": "rcedit.exe",
    }
