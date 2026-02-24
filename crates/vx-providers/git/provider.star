# provider.star - git provider
#
# Git - Distributed version control system
# Inheritance pattern: Level 2 (custom download_url for Windows portable Git)
#   - fetch_versions: custom (GitHub tags)
#   - download_url:   Windows-only portable Git from GitHub releases
#
# On macOS/Linux, git is typically pre-installed or available via system package manager.
# On Windows, vx can install portable Git from GitHub releases.

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

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
    {
        "name":        "git",
        "executable":  "git",
        "description": "Git version control",
        "aliases":     [],
        "priority":    100,
        "system_paths": [
            # Windows
            "C:/Program Files/Git/bin/git.exe",
            "C:/Program Files/Git/cmd/git.exe",
            # Unix
            "/usr/bin/git",
            "/usr/local/bin/git",
            "/opt/homebrew/bin/git",
        ],
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "git version"},
            {"command": "{executable} config --list", "name": "config_check", "expect_success": True},
        ],
        # Shells provided by Git (RFC 0038)
        # These can be launched with: vx git::git-bash, vx git::git-cmd
        "shells": [
            {"name": "git-bash", "path": "git-bash.exe"},
            {"name": "git-cmd",  "path": "git-cmd.exe"},
            {"name": "bash",     "path": "bin/bash.exe"},
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
# fetch_versions — GitHub tags (git-for-windows/git)
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("git-for-windows", "git")

# ---------------------------------------------------------------------------
# download_url — Windows-only portable Git
#
# Git for Windows asset naming: PortableGit-{version}-64-bit.7z.exe
# Note: .7z.exe is a self-extracting archive that can be run silently
# For vx we use the zip variant when available, otherwise skip macOS/Linux
# (they use system package manager)
#
# Windows portable: https://github.com/git-for-windows/git/releases/download/v{version}.windows.1/PortableGit-{version}-64-bit.7z.exe
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build the Git download URL.

    Only Windows is supported for direct download.
    macOS/Linux should use system package manager.

    Args:
        ctx:     Provider context
        version: Version string, e.g. "2.47.1"

    Returns:
        Download URL string, or None if not Windows
    """
    os   = ctx.platform.os
    arch = ctx.platform.arch

    if os != "windows":
        return None

    # Git for Windows uses a special tag format: v{version}.windows.1
    tag = "v{}.windows.1".format(version)

    if arch == "x64":
        asset = "PortableGit-{}-64-bit.7z.exe".format(version)
    elif arch == "x86":
        asset = "PortableGit-{}-32-bit.7z.exe".format(version)
    else:
        return None

    return github_asset_url("git-for-windows", "git", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(_ctx, _version):
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": ["bin/git.exe", "bin/git"],
    }

# ---------------------------------------------------------------------------
# system_install — package manager strategies
# ---------------------------------------------------------------------------

def system_install(_ctx):
    """Return system install strategies for git."""
    return [
        {"manager": "winget", "package": "Git.Git",  "priority": 70},
        {"manager": "choco",  "package": "git",       "priority": 80},
        {"manager": "brew",   "package": "git",       "priority": 90},
        {"manager": "apt",    "package": "git",       "priority": 90},
        {"manager": "dnf",    "package": "git",       "priority": 90},
        {"manager": "pacman", "package": "git",       "priority": 90},
    ]

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    os = ctx.platform.os
    if os == "windows":
        return [
            env_prepend("PATH", "{}/bin:{}/mingw64/bin:{}/usr/bin".format(
                ctx.install_dir, ctx.install_dir, ctx.install_dir
            )),
        ]
    return []


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/git"

def get_execute_path(ctx, _version):
    os = ctx.platform.os
    if os == "windows":
        return ctx.install_dir + "/git.exe"
    else:
        return ctx.install_dir + "/git"

def post_install(_ctx, _version):
    return None
