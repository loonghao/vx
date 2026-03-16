# provider.star - Shell Tools provider
#
# A collection of modern shell enhancement tools:
#   - starship:  The minimal, blazing-fast, and infinitely customizable prompt
#   - atuin:     Magical shell history (sync, search, and backup)
#   - yazi:      Blazing fast terminal file manager written in Rust
#
# Usage:
#   vx provider add ./examples/providers/shell-tools/provider.star
#   vx starship --version
#   vx atuin --version
#   vx yazi --version
#
# This example demonstrates:
#   1. Tools that enhance the shell experience (not built-in to vx)
#   2. Rust-compiled binaries with standard target-triple naming
#   3. Single-binary installs (no archive strip_prefix needed for some)

load("@vx//stdlib:provider.star",
     "runtime_def",
     "github_permissions")
load("@vx//stdlib:github.star",
     "make_fetch_versions",
     "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "shell-tools"
description = "Modern shell enhancement tools: starship, atuin, yazi"
homepage    = "https://github.com/loonghao/vx"
repository  = "https://github.com/loonghao/vx"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------
runtimes = [
    # starship — cross-shell prompt
    runtime_def("starship",
        description   = "The minimal, blazing-fast, and infinitely customizable prompt for any shell",
        aliases       = [],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "starship \\d+"},
        ],
    ),
    # atuin — shell history
    runtime_def("atuin",
        description   = "Magical shell history: sync, search, and backup shell history",
        aliases       = [],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "atuin \\d+"},
        ],
    ),
    # yazi — terminal file manager
    runtime_def("yazi",
        description   = "Blazing fast terminal file manager written in Rust, based on async I/O",
        aliases       = ["ya"],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "Yazi \\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------
permissions = github_permissions()

# ---------------------------------------------------------------------------
# Version fetching
# ---------------------------------------------------------------------------
def fetch_versions(ctx):
    """Fetch versions for the requested runtime from its GitHub repo."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "starship"

    repos = {
        "starship": ("starship-rs", "starship"),
        "atuin":    ("atuinsh",     "atuin"),
        "yazi":     ("sxyazi",      "yazi"),
        "ya":       ("sxyazi",      "yazi"),
    }
    entry = repos.get(runtime, repos["starship"])
    owner, repo = entry[0], entry[1]

    fetcher = make_fetch_versions(owner, repo)
    return fetcher(ctx)

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------
def _triple(ctx):
    """Return the Rust-style target triple for the current platform."""
    os   = ctx.platform.os
    arch = ctx.platform.arch
    triples = {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-gnu",
        "linux/arm64":  "aarch64-unknown-linux-gnu",
    }
    return triples.get("{}/{}".format(os, arch))

def _ext(ctx):
    return "zip" if ctx.platform.os == "windows" else "tar.gz"

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------
def download_url(ctx, version):
    """Build the download URL for the requested runtime and version."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "starship"
    triple  = _triple(ctx)
    ext     = _ext(ctx)

    if not triple:
        return None

    # ── starship ─────────────────────────────────────────────────────────────
    # Asset: starship-x86_64-unknown-linux-gnu.tar.gz
    # Tag:   v1.x.x
    if runtime == "starship":
        asset = "starship-{}.{}".format(triple, ext)
        return github_asset_url("starship-rs", "starship", "v" + version, asset)

    # ── atuin ────────────────────────────────────────────────────────────────
    # Asset: atuin-v18.x.x-x86_64-unknown-linux-gnu.tar.gz
    # Tag:   v18.x.x
    if runtime == "atuin":
        asset = "atuin-v{}-{}.{}".format(version, triple, ext)
        return github_asset_url("atuinsh", "atuin", "v" + version, asset)

    # ── yazi ─────────────────────────────────────────────────────────────────
    # Asset: yazi-x86_64-unknown-linux-gnu.zip  (always zip, even on Linux)
    # Tag:   v0.x.x
    if runtime in ("yazi", "ya"):
        # yazi uses zip for all platforms
        asset = "yazi-{}.zip".format(triple)
        return github_asset_url("sxyazi", "yazi", "v" + version, asset)

    return None

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------
def install_layout(ctx, version):
    """Return the archive extraction descriptor for the requested runtime."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "starship"
    os      = ctx.platform.os
    triple  = _triple(ctx)

    # ── starship: archive contains single binary at root ────────────────────
    if runtime == "starship":
        exe = "starship.exe" if os == "windows" else "starship"
        return {
            "__type":           "archive",
            "strip_prefix":     "",
            "executable_paths": [exe, "starship"],
        }

    # ── atuin: archive contains binary in a subdirectory ────────────────────
    if runtime == "atuin":
        # atuin-v{version}-{triple}/atuin
        strip = "atuin-v{}-{}".format(version, triple) if triple else ""
        exe   = "atuin.exe" if os == "windows" else "atuin"
        return {
            "__type":           "archive",
            "strip_prefix":     strip,
            "executable_paths": [exe, "atuin"],
        }

    # ── yazi: archive contains yazi + ya binaries ───────────────────────────
    if runtime in ("yazi", "ya"):
        # yazi-{triple}/yazi  and  yazi-{triple}/ya
        strip = "yazi-{}".format(triple) if triple else ""
        exe   = "yazi.exe" if os == "windows" else "yazi"
        return {
            "__type":           "archive",
            "strip_prefix":     strip,
            "executable_paths": [exe, "yazi"],
        }

    return {"__type": "archive", "strip_prefix": "", "executable_paths": []}

# ---------------------------------------------------------------------------
# Path queries
# ---------------------------------------------------------------------------
def store_root(ctx):
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "starship"
    return ctx.vx_home + "/store/" + runtime

def get_execute_path(ctx, _version):
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "starship"
    os      = ctx.platform.os

    exe_map = {
        "starship": "starship.exe" if os == "windows" else "starship",
        "atuin":    "atuin.exe"    if os == "windows" else "atuin",
        "yazi":     "yazi.exe"     if os == "windows" else "yazi",
        "ya":       "yazi.exe"     if os == "windows" else "yazi",
    }
    exe = exe_map.get(runtime, runtime)
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Environment
# ---------------------------------------------------------------------------
def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

# ---------------------------------------------------------------------------
# Dependencies
# ---------------------------------------------------------------------------
def deps(_ctx, _version):
    return []
