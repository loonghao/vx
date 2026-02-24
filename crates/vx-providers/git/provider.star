# provider.star - git provider
#
# Git - Distributed version control system
# Windows: portable Git from git-for-windows (7z.exe self-extracting)
# macOS/Linux: system package manager
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
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
    {
        "name":        "git",
        "executable":  "git",
        "description": "Git version control",
        "aliases":     [],
        "priority":    100,
        "system_paths": [
            "C:/Program Files/Git/bin/git.exe",
            "C:/Program Files/Git/cmd/git.exe",
            "/usr/bin/git",
            "/usr/local/bin/git",
            "/opt/homebrew/bin/git",
        ],
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "git version"},
            {"command": "{executable} config --list", "name": "config_check",
             "expect_success": True},
        ],
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

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("git-for-windows", "git")

# ---------------------------------------------------------------------------
# download_url — Windows-only portable Git (7z.exe self-extracting)
# macOS/Linux use system package manager
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    if ctx.platform.os != "windows":
        return None
    tag = "v{}.windows.1".format(version)
    if ctx.platform.arch == "x64":
        asset = "PortableGit-{}-64-bit.7z.exe".format(version)
    elif ctx.platform.arch == "x86":
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
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/git"

def get_execute_path(ctx, _version):
    if ctx.platform.os == "windows":
        return ctx.install_dir + "/git.exe"
    return ctx.install_dir + "/git"

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    if ctx.platform.os == "windows":
        return [
            env_prepend("PATH", "{}/bin:{}/mingw64/bin:{}/usr/bin".format(
                ctx.install_dir, ctx.install_dir, ctx.install_dir,
            )),
        ]
    return []

def deps(_ctx, _version):
    return []
