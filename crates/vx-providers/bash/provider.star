# provider.star - bash provider
#
# Bash: GNU Bourne Again SHell
# Cross-platform shell available on Linux/macOS natively,
# and on Windows via Git for Windows (Git Bash) or WSL.
#
# Installation strategy:
#   - Linux/macOS: system detection only (bash is pre-installed)
#   - Windows: download from Git for Windows (portable bash.exe)
#              or detect system bash (Git Bash, WSL, Cygwin)
#
# bash releases: https://github.com/git-for-windows/git/releases
# (Git for Windows bundles bash)

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "bash"
description = "GNU Bourne Again SHell - the standard Unix shell"
homepage    = "https://www.gnu.org/software/bash/"
repository  = "https://git.savannah.gnu.org/cgit/bash.git"
license     = "GPL-3.0"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "bash",
        "executable":  "bash",
        "description": "GNU Bash shell",
        "aliases":     ["sh"],
        "priority":    100,
        # Common system paths where bash may be found
        "system_paths": [
            # Linux/macOS
            "/bin/bash",
            "/usr/bin/bash",
            "/usr/local/bin/bash",
            # Windows - Git for Windows (Git Bash)
            "C:/Program Files/Git/bin/bash.exe",
            "C:/Program Files/Git/usr/bin/bash.exe",
            # Windows - Git for Windows (32-bit)
            "C:/Program Files (x86)/Git/bin/bash.exe",
            # Windows - Cygwin
            "C:/cygwin64/bin/bash.exe",
            "C:/cygwin/bin/bash.exe",
            # Windows - MSYS2
            "C:/msys64/usr/bin/bash.exe",
            "C:/msys32/usr/bin/bash.exe",
        ],
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "GNU bash"},
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
# fetch_versions
#
# On Windows, bash is bundled with Git for Windows.
# We fetch versions from the Git for Windows GitHub releases.
# On Linux/macOS, bash is a system tool — no version fetching needed.
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("git-for-windows", "git")

# ---------------------------------------------------------------------------
# download_url
#
# Windows: Download portable Git for Windows (which includes bash)
# Linux/macOS: bash is pre-installed, no download needed
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build the bash download URL.

    On Windows, bash is bundled with Git for Windows (PortableGit).
    On Linux/macOS, bash is a system tool and does not need to be downloaded.

    Args:
        ctx:     Provider context
        version: Version string, e.g. "2.47.1"

    Returns:
        Download URL string, or None if platform does not need download
    """
    os   = ctx.platform.os
    arch = ctx.platform.arch

    if os != "windows":
        # Linux/macOS: bash is pre-installed, use system detection
        return None

    # Windows: use Git for Windows PortableGit which includes bash
    # Asset format: PortableGit-{version}-64-bit.7z.exe
    # We use the MinGit (minimal Git) which is smaller and includes bash
    # MinGit-{version}-64-bit.zip
    if arch == "arm64":
        asset = "MinGit-{}-arm64.zip".format(version)
    else:
        asset = "MinGit-{}-64-bit.zip".format(version)

    tag = "v{}".format(version)
    return github_asset_url("git-for-windows", "git", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """Return the install layout descriptor.

    Args:
        ctx:     Provider context
        version: Version string

    Returns:
        Install layout descriptor dict
    """
    os = ctx.platform.os

    if os != "windows":
        # Linux/macOS: bash is a system tool, no install layout needed
        return None

    # Windows: MinGit zip contains bash in usr/bin/bash.exe
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": ["usr/bin/bash.exe", "bin/bash.exe", "bash.exe"],
    }

# ---------------------------------------------------------------------------
# system_install — package manager fallback
# ---------------------------------------------------------------------------

def system_install(ctx):
    """Return system package manager installation strategies.

    Args:
        ctx: Provider context

    Returns:
        Dict with strategies list
    """
    os = ctx.platform.os

    if os == "windows":
        return {
            "strategies": [
                # Git for Windows includes bash (most common on Windows)
                {"manager": "winget", "package": "Git.Git",        "priority": 95},
                {"manager": "choco",  "package": "git",            "priority": 80},
                {"manager": "scoop",  "package": "git",            "priority": 60},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                # macOS ships with an old bash (3.x); brew provides newer version
                {"manager": "brew", "package": "bash", "priority": 90},
            ],
        }
    elif os == "linux":
        return {
            "strategies": [
                {"manager": "apt",    "package": "bash", "priority": 90},
                {"manager": "dnf",    "package": "bash", "priority": 85},
                {"manager": "pacman", "package": "bash", "priority": 80},
            ],
        }
    return {}

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for bash."""
    return ctx.vx_home + "/store/bash"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx.platform.os
    if os == "windows":
        return ctx.install_dir + "/usr/bin/bash.exe"
    return "/bin/bash"

def post_install(_ctx, _version):
    """No post-install steps needed for bash."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    os = ctx.platform.os
    if os == "windows":
        # On Windows, prepend the bash bin directory to PATH
        return [env_prepend("PATH", ctx.install_dir + "/usr/bin")]
    return []

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return []
