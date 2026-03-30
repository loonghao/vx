# provider.star - biome (JS/TS linter and formatter)
#
# biome: Fast formatter and linter for JavaScript, TypeScript, JSX, JSON, CSS
# Releases: https://github.com/biomejs/biome/releases
# Asset format: biome-{os}-{arch}  (standalone binary, .exe on Windows)
# Tag format:   cli/v{version}
#
# Uses custom download_url because biome distributes standalone binaries
# (not archives) with unique os naming (win32, darwin, linux).

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "env_prepend", "path_fns",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:layout.star", "binary_layout")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "biome"
description = "biome - Fast formatter and linter for JS/TS/JSON/CSS"
homepage    = "https://biomejs.dev"
repository  = "https://github.com/biomejs/biome"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [runtime_def("biome", version_pattern="Version: \\d+")]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Platform mapping
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("win32", "x64"),
    "windows/arm64": ("win32", "arm64"),
    "macos/x64":     ("darwin", "x64"),
    "macos/arm64":   ("darwin", "arm64"),
    "linux/x64":     ("linux", "x64"),
    "linux/arm64":   ("linux", "arm64"),
}

# ---------------------------------------------------------------------------
# Provider functions
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix("biomejs", "biome", tag_prefix="cli/v")

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return None
    os_str, arch_str = platform
    exe = ".exe" if ctx.platform.os == "windows" else ""
    return "https://github.com/biomejs/biome/releases/download/cli/v{}/biome-{}-{}{}".format(
        version, os_str, arch_str, exe)

install_layout = binary_layout("biome")

paths = path_fns("biome")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
