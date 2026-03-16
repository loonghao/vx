# provider.star - Git Tools provider
#
# Terminal UI tools for Git and Docker workflows:
#   - lazygit:    Simple terminal UI for git commands
#   - lazydocker: Simple terminal UI for docker and docker-compose
#
# Usage:
#   vx provider add ./examples/providers/git-tools/provider.star
#   vx lazygit
#   vx lazydocker
#
# This example demonstrates:
#   1. Tools from the same author (jesseduffield) sharing similar asset naming
#   2. OS names with capital letters (Linux, Darwin, Windows)
#   3. Single-binary archives (no strip_prefix needed)

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
name        = "git-tools"
description = "Terminal UI tools for Git and Docker: lazygit, lazydocker"
homepage    = "https://github.com/jesseduffield"
repository  = "https://github.com/jesseduffield/lazygit"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------
runtimes = [
    # lazygit — terminal UI for git
    runtime_def("lazygit",
        description   = "Simple terminal UI for git commands",
        aliases       = ["lg"],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "version="},
        ],
    ),
    # lazydocker — terminal UI for docker
    runtime_def("lazydocker",
        description   = "Simple terminal UI for docker and docker-compose",
        aliases       = ["lzd"],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "Version:"},
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
    """Fetch versions from the respective GitHub repo."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "lazygit"

    repos = {
        "lazygit":    ("jesseduffield", "lazygit"),
        "lg":         ("jesseduffield", "lazygit"),
        "lazydocker": ("jesseduffield", "lazydocker"),
        "lzd":        ("jesseduffield", "lazydocker"),
    }
    entry = repos.get(runtime, repos["lazygit"])
    owner, repo = entry[0], entry[1]

    fetcher = make_fetch_versions(owner, repo)
    return fetcher(ctx)

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------
def _os_name(ctx):
    """Return the OS name as used in jesseduffield release assets (capitalized)."""
    os_map = {
        "windows": "Windows",
        "macos":   "Darwin",
        "linux":   "Linux",
    }
    return os_map.get(ctx.platform.os, "Linux")

def _arch_name(ctx):
    """Return the architecture name as used in jesseduffield release assets."""
    arch_map = {
        "x64":   "x86_64",
        "arm64": "arm64",
        "arm":   "armv6",
    }
    return arch_map.get(ctx.platform.arch, "x86_64")

def _ext(ctx):
    return "zip" if ctx.platform.os == "windows" else "tar.gz"

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------
def download_url(ctx, version):
    """Build the download URL for lazygit or lazydocker."""
    runtime  = ctx.runtime_name if hasattr(ctx, "runtime_name") else "lazygit"
    os_name  = _os_name(ctx)
    arch     = _arch_name(ctx)
    ext      = _ext(ctx)

    # lazygit: lazygit_{version}_{OS}_{ARCH}.tar.gz
    # e.g.   : lazygit_0.44.1_Linux_x86_64.tar.gz
    if runtime in ("lazygit", "lg"):
        asset = "lazygit_{}_{}_{}.{}".format(version, os_name, arch, ext)
        return github_asset_url("jesseduffield", "lazygit", "v" + version, asset)

    # lazydocker: lazydocker_{version}_{OS}_{ARCH}.tar.gz
    # e.g.      : lazydocker_0.24.1_Linux_x86_64.tar.gz
    if runtime in ("lazydocker", "lzd"):
        asset = "lazydocker_{}_{}_{}.{}".format(version, os_name, arch, ext)
        return github_asset_url("jesseduffield", "lazydocker", "v" + version, asset)

    return None

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------
def install_layout(ctx, _version):
    """Both tools ship as a single binary inside a flat archive."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "lazygit"
    os      = ctx.platform.os

    if runtime in ("lazygit", "lg"):
        exe = "lazygit.exe" if os == "windows" else "lazygit"
        return {
            "__type":           "archive",
            "strip_prefix":     "",          # flat archive, no subdirectory
            "executable_paths": [exe, "lazygit"],
        }

    if runtime in ("lazydocker", "lzd"):
        exe = "lazydocker.exe" if os == "windows" else "lazydocker"
        return {
            "__type":           "archive",
            "strip_prefix":     "",
            "executable_paths": [exe, "lazydocker"],
        }

    return {"__type": "archive", "strip_prefix": "", "executable_paths": []}

# ---------------------------------------------------------------------------
# Path queries
# ---------------------------------------------------------------------------
def store_root(ctx):
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "lazygit"
    return ctx.vx_home + "/store/" + runtime

def get_execute_path(ctx, _version):
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "lazygit"
    os      = ctx.platform.os

    exe_map = {
        "lazygit":    "lazygit.exe"    if os == "windows" else "lazygit",
        "lg":         "lazygit.exe"    if os == "windows" else "lazygit",
        "lazydocker": "lazydocker.exe" if os == "windows" else "lazydocker",
        "lzd":        "lazydocker.exe" if os == "windows" else "lazydocker",
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
    # lazydocker requires docker to be installed (system dependency, not vx-managed)
    return []
