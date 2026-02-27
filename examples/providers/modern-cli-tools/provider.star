# provider.star - Modern CLI Tools provider
#
# A collection of modern interactive CLI tools:
#   - fzf:    General-purpose command-line fuzzy finder
#   - delta:  Syntax-highlighting pager for git and diff output
#   - zoxide: A smarter cd command (tracks your most used directories)
#
# Usage:
#   vx provider add ./examples/providers/modern-cli-tools/provider.star
#   vx fzf --version
#   vx delta --version
#   vx zoxide --version
#
# This example demonstrates:
#   1. Multiple runtimes in one provider.star
#   2. Per-tool GitHub release asset naming
#   3. system_install fallback via brew / winget / scoop
#   4. deps() — zoxide has no deps; fzf/delta are standalone

load("@vx//stdlib:provider.star",
     "runtime_def",
     "github_permissions",
     "multi_platform_install",
     "winget_install", "scoop_install",
     "brew_install")
load("@vx//stdlib:github.star",
     "make_fetch_versions",
     "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "modern-cli-tools"
description = "Modern interactive CLI tools: fzf, delta, zoxide"
homepage    = "https://github.com/loonghao/vx"
repository  = "https://github.com/loonghao/vx"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------
runtimes = [
    # fzf — fuzzy finder
    runtime_def("fzf",
        description   = "A command-line fuzzy finder",
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+"},
        ],
    ),
    # delta — git diff pager
    runtime_def("delta",
        description   = "A syntax-highlighting pager for git, diff, and grep output",
        aliases       = ["git-delta"],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "delta \\d+"},
        ],
    ),
    # zoxide — smarter cd
    runtime_def("zoxide",
        description   = "A smarter cd command that learns your habits",
        aliases       = ["z"],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "zoxide \\d+"},
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
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "fzf"

    repos = {
        "fzf":       ("junegunn",  "fzf"),
        "delta":     ("dandavison", "delta"),
        "git-delta": ("dandavison", "delta"),
        "zoxide":    ("ajeetdsouza", "zoxide"),
        "z":         ("ajeetdsouza", "zoxide"),
    }
    entry = repos.get(runtime, repos["fzf"])
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
        "linux/x64":    "x86_64-unknown-linux-musl",
        "linux/arm64":  "aarch64-unknown-linux-musl",
    }
    return triples.get("{}/{}".format(os, arch))

def _ext(ctx):
    return "zip" if ctx.platform.os == "windows" else "tar.gz"

# ---------------------------------------------------------------------------
# download_url — dispatches per runtime
# ---------------------------------------------------------------------------
def download_url(ctx, version):
    """Build the download URL for the requested runtime and version."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "fzf"
    os      = ctx.platform.os
    arch    = ctx.platform.arch
    triple  = _triple(ctx)
    ext     = _ext(ctx)

    # ── fzf ─────────────────────────────────────────────────────────────────
    # Asset: fzf-{version}-{os}_{arch}.{ext}
    # e.g.  fzf-0.54.3-linux_amd64.tar.gz
    if runtime == "fzf":
        fzf_os   = {"windows": "windows", "macos": "darwin", "linux": "linux"}.get(os)
        fzf_arch = {"x64": "amd64", "arm64": "arm64"}.get(arch)
        if not fzf_os or not fzf_arch:
            return None
        asset = "fzf-{}-{}_{}.{}".format(version, fzf_os, fzf_arch, ext)
        return github_asset_url("junegunn", "fzf", "v" + version, asset)

    # ── delta ────────────────────────────────────────────────────────────────
    # Asset: delta-{version}-{triple}.{ext}
    # e.g.  delta-0.17.0-x86_64-unknown-linux-musl.tar.gz
    if runtime in ("delta", "git-delta"):
        if not triple:
            return None
        asset = "delta-{}-{}.{}".format(version, triple, ext)
        return github_asset_url("dandavison", "delta", version, asset)

    # ── zoxide ───────────────────────────────────────────────────────────────
    # Asset: zoxide-{version}-{triple}.{ext}
    # e.g.  zoxide-0.9.4-x86_64-unknown-linux-musl.tar.gz
    if runtime in ("zoxide", "z"):
        if not triple:
            return None
        asset = "zoxide-{}-{}.{}".format(version, triple, ext)
        return github_asset_url("ajeetdsouza", "zoxide", "v" + version, asset)

    return None

# ---------------------------------------------------------------------------
# install_layout — dispatches per runtime
# ---------------------------------------------------------------------------
def install_layout(ctx, version):
    """Return the archive extraction descriptor for the requested runtime."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "fzf"
    os      = ctx.platform.os
    arch    = ctx.platform.arch
    triple  = _triple(ctx)

    # ── fzf ─────────────────────────────────────────────────────────────────
    if runtime == "fzf":
        exe = "fzf.exe" if os == "windows" else "fzf"
        # fzf archive has no subdirectory prefix
        return {
            "__type":           "archive",
            "strip_prefix":     "",
            "executable_paths": [exe, "fzf"],
        }

    # ── delta ────────────────────────────────────────────────────────────────
    if runtime in ("delta", "git-delta"):
        strip = "delta-{}-{}".format(version, triple) if triple else ""
        exe   = "delta.exe" if os == "windows" else "delta"
        return {
            "__type":           "archive",
            "strip_prefix":     strip,
            "executable_paths": [exe, "delta"],
        }

    # ── zoxide ───────────────────────────────────────────────────────────────
    if runtime in ("zoxide", "z"):
        # zoxide archive has no subdirectory prefix
        exe = "zoxide.exe" if os == "windows" else "zoxide"
        return {
            "__type":           "archive",
            "strip_prefix":     "",
            "executable_paths": [exe, "zoxide"],
        }

    return {"__type": "archive", "strip_prefix": "", "executable_paths": []}

# ---------------------------------------------------------------------------
# system_install — package manager fallback
# ---------------------------------------------------------------------------
system_install = multi_platform_install(
    windows_strategies = [
        winget_install("junegunn.fzf",        priority = 90),   # fzf
        scoop_install("fzf",                  priority = 70),
        winget_install("dandavison.delta",    priority = 90),   # delta
        scoop_install("delta",                priority = 70),
        winget_install("ajeetdsouza.zoxide",  priority = 90),   # zoxide
        scoop_install("zoxide",               priority = 70),
    ],
    macos_strategies = [
        brew_install("fzf",    priority = 90),
        brew_install("git-delta", priority = 90),
        brew_install("zoxide", priority = 90),
    ],
    linux_strategies = [
        brew_install("fzf",    priority = 70),
        brew_install("git-delta", priority = 70),
        brew_install("zoxide", priority = 70),
    ],
)

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------
def store_root(ctx):
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "fzf"
    return ctx.vx_home + "/store/" + runtime

def get_execute_path(ctx, _version):
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "fzf"
    os      = ctx.platform.os

    exe_map = {
        "fzf":       "fzf.exe"    if os == "windows" else "fzf",
        "delta":     "delta.exe"  if os == "windows" else "delta",
        "git-delta": "delta.exe"  if os == "windows" else "delta",
        "zoxide":    "zoxide.exe" if os == "windows" else "zoxide",
        "z":         "zoxide.exe" if os == "windows" else "zoxide",
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
# Dependencies — all three tools are standalone, no runtime deps
# ---------------------------------------------------------------------------
def deps(_ctx, _version):
    return []
