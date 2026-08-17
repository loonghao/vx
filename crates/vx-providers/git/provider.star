# provider.star - Git provider
#
# Git - Distributed version control system
# Windows: full Git for Windows tar archive
# macOS/Linux: system package manager
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def",
     "github_permissions",
     "path_fns",
     "system_install_strategies", "pkg_strategy")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "git"
description = "Git - Distributed version control system"
homepage    = "https://git-scm.com"
repository  = "https://github.com/git-for-windows/git"
license     = "GPL-2.0"
ecosystem   = "git"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("git",
        description = "Git version control",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("git-for-windows", "git")

# ---------------------------------------------------------------------------
# download_url — Windows-only full Git archive
# macOS/Linux use system package manager
#
# Version format: "{base}.windows.{N}" (e.g. "2.53.0.windows.2")
# Tag:   "v{base}.windows.{N}"  (e.g. "v2.53.0.windows.2")
# Asset version:
#   N=1 → "{base}"       (e.g. "2.53.0")
#   N>1 → "{base}.{N}"   (e.g. "2.53.0.2")
#
# The full tar.bz2 distribution includes bash for #!/usr/bin/env bash hooks.
# Unlike PortableGit's .7z.exe, it is also a regular archive vx can extract
# without handling a self-extracting PE wrapper.
# ---------------------------------------------------------------------------

def _parse_git_version(version):
    """Parse git-for-windows version string.

    Returns (base, windows_n) where base is the semver part and
    windows_n is the integer patch suffix (1, 2, …).

    Accepts both:
      "2.53.0.windows.2"  → ("2.53.0", 2)
      "2.53.0"            → ("2.53.0", 1)  # treat plain version as .windows.1
    """
    marker = ".windows."
    idx = version.find(marker)
    if idx >= 0:
        base = version[:idx]
        n_str = version[idx + len(marker):]
        n = int(n_str) if n_str.isdigit() else 1
    else:
        base = version
        n = 1
    return base, n

def download_url(ctx, version):
    if ctx.platform.os != "windows":
        return None

    base, n = _parse_git_version(version)

    # The GitHub tag is always "v{base}.windows.{N}"
    tag = "v{}.windows.{}".format(base, n)

    # Asset filename uses "{base}" for .windows.1, "{base}.{N}" for .windows.N>1
    asset_ver = "{}.{}".format(base, n) if n > 1 else base

    if ctx.platform.arch == "x64":
        asset = "Git-{}-64-bit.tar.bz2".format(asset_ver)
    elif ctx.platform.arch == "arm64":
        asset = "Git-{}-arm64.tar.bz2".format(asset_ver)
    else:
        return None

    return github_asset_url("git-for-windows", "git", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

# Full Git tar.bz2 extracted layout:
#   <install_dir>/
#     bin/git.exe          ← full-distribution entry point and migration marker
#     cmd/git.exe          ← cmd-style wrapper
#     mingw64/bin/git.exe  ← real MinGW git binary
#     mingw64/bin/...      ← other git tools
#     usr/bin/bash.exe     ← interpreter for common Git hooks
#
# The archive is extracted directly into install_dir with no top-level directory,
# so strip_prefix="" (auto-detect) will NOT attempt to strip any prefix.
# MinGit lacks bin/git.exe and usr/bin/bash.exe. Requiring both full-distribution
# paths lets vx repair older cached MinGit installs automatically.
def install_layout(ctx, _version):
    if ctx.platform.os == "windows":
        return {
            "type":             "archive",
            "strip_prefix":     "",
            "executable_paths": ["bin/git.exe"],
            "required_paths":   ["usr/bin/bash.exe"],
        }
    # Non-Windows: plain archive (or system install — download_url returns None)
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": ["bin/git", "git"],
    }

# ---------------------------------------------------------------------------
# system_install — package manager strategies
# ---------------------------------------------------------------------------

# git is cross-platform: all managers, no platform restriction
system_install = system_install_strategies([
    pkg_strategy("winget", "Git.Git", priority = 70),
    pkg_strategy("choco",  "git",     priority = 80),
    pkg_strategy("brew",   "git",     priority = 90),
    pkg_strategy("apt",    "git",     priority = 90),
    pkg_strategy("dnf",    "git",     priority = 90),
    pkg_strategy("pacman", "git",     priority = 90),
])

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

_paths     = path_fns("git")
store_root = _paths["store_root"]

def get_execute_path(ctx, _version):
    """Return the path to the git executable inside the install dir.

    Git for Windows extracts to a directory tree; the canonical
    entry point is bin/git.exe. Its presence also distinguishes the full
    distribution from older MinGit installs that do not include Bash.

    On non-Windows the tool is managed by the system package manager,
    so install_dir points to the vx store where we placed the binary.
    """
    if ctx.platform.os == "windows":
        return ctx.install_dir + "/bin/git.exe"
    return ctx.install_dir + "/bin/git"

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    if ctx.platform.os == "windows":
        # Prepend each directory separately so env_prepend uses the correct
        # OS-specific PATH separator (';' on Windows, ':' on Unix).
        # Order matters: later prepends appear earlier in PATH, so list the
        # most specific path last (it ends up first in PATH).
        return [
            env_prepend("PATH", ctx.install_dir + "/usr/bin"),
            env_prepend("PATH", ctx.install_dir + "/bin"),
            env_prepend("PATH", ctx.install_dir + "/mingw64/bin"),
            env_prepend("PATH", ctx.install_dir + "/cmd"),
        ]
    return []

def deps(_ctx, _version):
    return []
