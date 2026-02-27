# provider.star - Unix Tools provider
#
# A collection of modern Unix command-line tools:
#   - jq:      Lightweight and flexible command-line JSON processor
#   - ripgrep: Fast regex search tool (rg)
#   - fd:      Simple, fast alternative to 'find'
#   - bat:     A cat clone with syntax highlighting
#
# Usage:
#   vx provider add ./examples/providers/unix-tools/provider.star
#   vx jq --version
#   vx rg --version
#   vx fd --version
#   vx bat --version
#
# This example demonstrates:
#   1. A single provider.star exposing MULTIPLE runtimes
#   2. Per-tool platform-specific asset naming conventions
#   3. Using stdlib helpers (make_fetch_versions, github_asset_url)
#   4. archive_layout / path_fns for zero-boilerplate path queries

load("@vx//stdlib:provider.star",
     "runtime_def",
     "github_permissions",
     "archive_layout",
     "path_fns",
     "path_env_fns")
load("@vx//stdlib:github.star",
     "make_fetch_versions",
     "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "unix-tools"
description = "Modern Unix command-line tools: jq, ripgrep, fd, bat"
homepage    = "https://github.com/loonghao/vx"
repository  = "https://github.com/loonghao/vx"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# Each tool is a separate runtime within this provider.
# ---------------------------------------------------------------------------
runtimes = [
    # jq — JSON processor
    runtime_def("jq",
        description   = "Lightweight and flexible command-line JSON processor",
        aliases       = ["jq-tool"],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "jq-\\d+"},
        ],
    ),
    # ripgrep — fast grep replacement
    runtime_def("ripgrep",
        description   = "Recursively searches directories for a regex pattern",
        aliases       = ["rg"],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "ripgrep \\d+"},
        ],
    ),
    # fd — fast find replacement
    runtime_def("fd",
        description   = "Simple, fast and user-friendly alternative to 'find'",
        aliases       = ["fd-find"],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "fd \\d+"},
        ],
    ),
    # bat — cat with syntax highlighting
    runtime_def("bat",
        description   = "A cat clone with syntax highlighting and Git integration",
        aliases       = ["batcat"],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "bat \\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------
permissions = github_permissions()

# ---------------------------------------------------------------------------
# Version fetching — each tool has its own GitHub repo
# ---------------------------------------------------------------------------
def fetch_versions(ctx):
    """Fetch versions for the requested runtime from its GitHub repo."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "jq"

    repos = {
        "jq":      ("jqlang",       "jq"),
        "ripgrep": ("BurntSushi",   "ripgrep"),
        "rg":      ("BurntSushi",   "ripgrep"),
        "fd":      ("sharkdp",      "fd"),
        "fd-find": ("sharkdp",      "fd"),
        "bat":     ("sharkdp",      "bat"),
        "batcat":  ("sharkdp",      "bat"),
    }
    entry = repos.get(runtime, repos["jq"])
    owner, repo = entry[0], entry[1]

    load("@vx//stdlib:github.star", "make_fetch_versions")
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
        "linux/x64":    "x86_64-unknown-linux-musl",
        "linux/arm64":  "aarch64-unknown-linux-gnu",
    }
    return triples.get("{}/{}".format(os, arch))

def _ext(ctx):
    return "zip" if ctx.platform.os == "windows" else "tar.gz"

# ---------------------------------------------------------------------------
# download_url — dispatches per runtime
# ---------------------------------------------------------------------------
def download_url(ctx, version):
    """Build the download URL for the requested runtime and version."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "jq"
    os      = ctx.platform.os
    arch    = ctx.platform.arch
    triple  = _triple(ctx)
    ext     = _ext(ctx)

    # ── jq ──────────────────────────────────────────────────────────────────
    if runtime in ("jq", "jq-tool"):
        # jq uses "jq-X.Y" tags; asset: jq-{os} or jq-{os}-amd64
        if os == "windows":
            asset = "jq-windows-amd64.exe" if arch == "x64" else "jq-windows-arm64.exe"
        elif os == "macos":
            asset = "jq-macos-amd64" if arch == "x64" else "jq-macos-arm64"
        else:
            asset = "jq-linux-amd64" if arch == "x64" else "jq-linux-arm64"
        return github_asset_url("jqlang", "jq", "jq-" + version, asset)

    # ── ripgrep ─────────────────────────────────────────────────────────────
    if runtime in ("ripgrep", "rg"):
        if not triple:
            return None
        # ripgrep tags have NO 'v' prefix
        asset = "ripgrep-{}-{}.{}".format(version, triple, ext)
        return github_asset_url("BurntSushi", "ripgrep", version, asset)

    # ── fd ──────────────────────────────────────────────────────────────────
    if runtime in ("fd", "fd-find"):
        if not triple:
            return None
        asset = "fd-v{}-{}.{}".format(version, triple, ext)
        return github_asset_url("sharkdp", "fd", "v" + version, asset)

    # ── bat ─────────────────────────────────────────────────────────────────
    if runtime in ("bat", "batcat"):
        if not triple:
            return None
        asset = "bat-v{}-{}.{}".format(version, triple, ext)
        return github_asset_url("sharkdp", "bat", "v" + version, asset)

    return None

# ---------------------------------------------------------------------------
# install_layout — dispatches per runtime
# ---------------------------------------------------------------------------
def install_layout(ctx, version):
    """Return the archive extraction descriptor for the requested runtime."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "jq"
    os      = ctx.platform.os
    triple  = _triple(ctx)

    # ── jq: single binary (no archive) ──────────────────────────────────────
    if runtime in ("jq", "jq-tool"):
        exe = "jq.exe" if os == "windows" else "jq"
        return {
            "type":             "binary",
            "executable_name":  exe,
        }

    # ── ripgrep ─────────────────────────────────────────────────────────────
    if runtime in ("ripgrep", "rg"):
        strip = "ripgrep-{}-{}".format(version, triple) if triple else ""
        exe   = "rg.exe" if os == "windows" else "rg"
        return {
            "type":             "archive",
            "strip_prefix":     strip,
            "executable_paths": [exe, "rg"],
        }

    # ── fd ──────────────────────────────────────────────────────────────────
    if runtime in ("fd", "fd-find"):
        strip = "fd-v{}-{}".format(version, triple) if triple else ""
        exe   = "fd.exe" if os == "windows" else "fd"
        return {
            "type":             "archive",
            "strip_prefix":     strip,
            "executable_paths": [exe, "fd"],
        }

    # ── bat ─────────────────────────────────────────────────────────────────
    if runtime in ("bat", "batcat"):
        strip = "bat-v{}-{}".format(version, triple) if triple else ""
        exe   = "bat.exe" if os == "windows" else "bat"
        return {
            "type":             "archive",
            "strip_prefix":     strip,
            "executable_paths": [exe, "bat"],
        }

    return {"type": "archive", "strip_prefix": "", "executable_paths": []}

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------
def store_root(ctx):
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "jq"
    return ctx.vx_home + "/store/" + runtime

def get_execute_path(ctx, _version):
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "jq"
    os      = ctx.platform.os

    exe_map = {
        "jq":      "jq.exe"  if os == "windows" else "jq",
        "jq-tool": "jq.exe"  if os == "windows" else "jq",
        "ripgrep": "rg.exe"  if os == "windows" else "rg",
        "rg":      "rg.exe"  if os == "windows" else "rg",
        "fd":      "fd.exe"  if os == "windows" else "fd",
        "fd-find": "fd.exe"  if os == "windows" else "fd",
        "bat":     "bat.exe" if os == "windows" else "bat",
        "batcat":  "bat.exe" if os == "windows" else "bat",
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
