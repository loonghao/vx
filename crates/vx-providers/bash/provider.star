# provider.star - bash provider
#
# Linux/macOS: system detection only (pre-installed)
# Windows: MinGit from git-for-windows (includes bash)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "system_install_strategies", "winget_install", "choco_install",
     "scoop_install", "brew_install", "apt_install", "dnf_install",
     "pacman_install")
load("@vx//stdlib:github.star",   "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",      "env_prepend")

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
    runtime_def("bash",
        aliases      = ["sh"],
        system_paths = [
            "/bin/bash",
            "/usr/bin/bash",
            "/usr/local/bin/bash",
            "C:/Program Files/Git/bin/bash.exe",
            "C:/Program Files/Git/usr/bin/bash.exe",
            "C:/Program Files (x86)/Git/bin/bash.exe",
            "C:/cygwin64/bin/bash.exe",
            "C:/cygwin/bin/bash.exe",
            "C:/msys64/usr/bin/bash.exe",
            "C:/msys32/usr/bin/bash.exe",
        ],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "GNU bash"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — git-for-windows (bundles bash on Windows)
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("git-for-windows", "git")

# ---------------------------------------------------------------------------
# download_url — Windows: MinGit; Linux/macOS: system
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    if ctx.platform.os != "windows":
        return None
    if ctx.platform.arch == "arm64":
        asset = "MinGit-{}-arm64.zip".format(version)
    else:
        asset = "MinGit-{}-64-bit.zip".format(version)
    return github_asset_url("git-for-windows", "git", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    if ctx.platform.os != "windows":
        return None
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": ["usr/bin/bash.exe", "bin/bash.exe", "bash.exe"],
    }

# ---------------------------------------------------------------------------
# system_install
# ---------------------------------------------------------------------------

# ---------------------------------------------------------------------------
# system_install
# ---------------------------------------------------------------------------
# NOTE: Use explicit function (not multi_platform_install closure) so that
# parse_system_install_strategies can reliably detect and call it.

def system_install(ctx):
    os = ctx.platform.os
    if os == "windows":
        return system_install_strategies([
            winget_install("Git.Git", priority = 95),
            choco_install("git",      priority = 80),
            scoop_install("git",      priority = 60),
        ])
    elif os == "macos":
        return system_install_strategies([
            brew_install("bash"),
        ])
    elif os == "linux":
        return system_install_strategies([
            apt_install("bash",    priority = 90),
            dnf_install("bash",    priority = 85),
            pacman_install("bash", priority = 80),
        ])
    return {}

# ---------------------------------------------------------------------------
# Path queries + environment
# Note: bash on Linux/macOS uses system path, not vx store
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/bash"

def get_execute_path(ctx, _version):
    if ctx.platform.os == "windows":
        return ctx.install_dir + "/usr/bin/bash.exe"
    return "/bin/bash"

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    if ctx.platform.os == "windows":
        return [env_prepend("PATH", ctx.install_dir + "/usr/bin")]
    return []

def deps(_ctx, _version):
    return []
