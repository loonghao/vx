# provider.star - Network Tools provider
#
# A collection of network / HTTP / GitHub CLI tools:
#   - gh:  GitHub's official command-line tool
#   - xh:  Friendly, fast tool for sending HTTP requests (HTTPie in Rust)
#
# Usage:
#   vx provider add ./examples/providers/network-tools/provider.star
#   vx gh --version
#   vx xh --version
#
# This example demonstrates:
#   1. Mixed asset naming conventions (gh uses os_arch, xh uses Rust triples)
#   2. Windows uses .zip while Linux/macOS use .tar.gz
#   3. gh has a nested bin/ directory inside the archive
#   4. xh is a single binary inside a versioned folder

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
name        = "network-tools"
description = "Network & HTTP tools: gh (GitHub CLI), xh (HTTPie in Rust)"
homepage    = "https://github.com/loonghao/vx"
repository  = "https://github.com/loonghao/vx"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------
runtimes = [
    # gh — GitHub CLI
    runtime_def("gh",
        description   = "GitHub's official command line tool",
        aliases       = ["github-cli"],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "gh version \\d+"},
        ],
    ),
    # xh — friendly, fast HTTP requests (HTTPie rewrite in Rust)
    runtime_def("xh",
        description   = "Friendly, fast tool for sending HTTP requests (HTTPie in Rust)",
        aliases       = ["xhs"],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "xh \\d+"},
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
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "gh"

    repos = {
        "gh":         ("cli",     "cli"),
        "github-cli": ("cli",     "cli"),
        "xh":         ("ducaale", "xh"),
        "xhs":        ("ducaale", "xh"),
    }
    entry = repos.get(runtime, repos["gh"])
    owner, repo = entry[0], entry[1]

    fetcher = make_fetch_versions(owner, repo)
    return fetcher(ctx)

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------
def _os_name(ctx):
    """Return the OS name as used in gh release assets."""
    os = ctx.platform.os
    if os == "windows":
        return "windows"
    elif os == "macos":
        return "macOS"
    else:
        return "linux"

def _arch_name(ctx):
    """Return the arch name as used in gh release assets."""
    arch = ctx.platform.arch
    if arch == "arm64":
        return "arm64"
    else:
        return "amd64"

def _xh_triple(ctx):
    """Return the Rust target triple used by xh release assets."""
    os   = ctx.platform.os
    arch = ctx.platform.arch
    triples = {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-musl",
        "linux/arm64":  "aarch64-unknown-linux-musl",
    }
    return triples.get("{}/{}".format(os, arch))

def _ext(ctx):
    """Return archive extension: zip on Windows, tar.gz elsewhere."""
    return "zip" if ctx.platform.os == "windows" else "tar.gz"

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------
def download_url(ctx, version):
    """Build the download URL for the requested runtime and version."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "gh"
    os      = ctx.platform.os
    ext     = _ext(ctx)

    # ── gh ──────────────────────────────────────────────────────────────────
    # Asset pattern: gh_{version}_{os}_{arch}.{ext}
    # Tag:           v{version}
    # Example:       gh_2.67.0_linux_amd64.tar.gz
    #                gh_2.67.0_windows_amd64.zip
    #                gh_2.67.0_macOS_arm64.tar.gz
    if runtime in ("gh", "github-cli"):
        os_str   = _os_name(ctx)
        arch_str = _arch_name(ctx)
        asset    = "gh_{}_{}_{}.{}".format(version, os_str, arch_str, ext)
        # gh uses "v{version}" tags
        return github_asset_url("cli", "cli", "v" + version, asset)

    # ── xh ──────────────────────────────────────────────────────────────────
    # Asset pattern: xh-v{version}-{triple}.{ext}
    # Tag:           v{version}
    # Example:       xh-v0.22.2-x86_64-unknown-linux-musl.tar.gz
    #                xh-v0.22.2-x86_64-pc-windows-msvc.zip
    if runtime in ("xh", "xhs"):
        triple = _xh_triple(ctx)
        if not triple:
            return None
        asset = "xh-v{}-{}.{}".format(version, triple, ext)
        return github_asset_url("ducaale", "xh", "v" + version, asset)

    return None

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------
def install_layout(ctx, version):
    """Return the archive extraction descriptor for the requested runtime."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "gh"
    os      = ctx.platform.os

    # ── gh ──────────────────────────────────────────────────────────────────
    # Archive structure: gh_{version}_{os}_{arch}/bin/gh[.exe]
    if runtime in ("gh", "github-cli"):
        os_str   = _os_name(ctx)
        arch_str = _arch_name(ctx)
        strip    = "gh_{}_{}_{}".format(version, os_str, arch_str)
        exe      = "bin/gh.exe" if os == "windows" else "bin/gh"
        return {
            "type":             "archive",
            "strip_prefix":     strip,
            "executable_paths": [exe, "gh"],
        }

    # ── xh ──────────────────────────────────────────────────────────────────
    # Archive structure: xh-v{version}-{triple}/xh[.exe]
    if runtime in ("xh", "xhs"):
        triple = _xh_triple(ctx)
        strip  = "xh-v{}-{}".format(version, triple) if triple else ""
        exe    = "xh.exe" if os == "windows" else "xh"
        return {
            "type":             "archive",
            "strip_prefix":     strip,
            "executable_paths": [exe, "xh"],
        }

    return {"type": "archive", "strip_prefix": "", "executable_paths": []}

# ---------------------------------------------------------------------------
# Path queries
# ---------------------------------------------------------------------------
def store_root(ctx):
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "gh"
    return ctx.vx_home + "/store/" + runtime

def get_execute_path(ctx, _version):
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "gh"
    os      = ctx.platform.os

    exe_map = {
        "gh":         "gh.exe"  if os == "windows" else "gh",
        "github-cli": "gh.exe"  if os == "windows" else "gh",
        "xh":         "xh.exe"  if os == "windows" else "xh",
        "xhs":        "xh.exe"  if os == "windows" else "xh",
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
