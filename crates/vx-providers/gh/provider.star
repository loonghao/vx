# provider.star - GitHub CLI (gh) provider
#
# gh uses a unique naming scheme:
#   - macOS: "macOS" (capital M), Windows/Linux: lowercase
#   - All platforms use zip except Linux (tar.gz)
#   - Archive has a top-level dir on macOS/Linux: gh_{version}_{os}_{arch}/
#
# Uses runtime_def + github_permissions from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "gh"
description = "GitHub CLI - command line tool that brings GitHub to your terminal"
homepage    = "https://cli.github.com/"
repository  = "https://github.com/cli/cli"
license     = "MIT"
ecosystem   = "devtools"
aliases     = ["github-cli", "github"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("gh",
        aliases         = ["github-cli", "github"],
        version_pattern = "gh version",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["objects.githubusercontent.com"])

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("cli", "cli")

# ---------------------------------------------------------------------------
# Platform helpers
#
# gh uses:
#   - "macOS" (capital M) for macOS
#   - "linux" / "windows" (lowercase) for others
#   - "amd64" / "arm64" for arch
#   - zip for Windows + macOS, tar.gz for Linux
# ---------------------------------------------------------------------------

def _gh_platform(ctx):
    os   = ctx.platform.os
    arch = ctx.platform.arch

    arch_map = {"x64": "amd64", "arm64": "arm64", "x86": "386"}
    arch_name = arch_map.get(arch)
    if not arch_name:
        return None

    if os == "windows":
        return ("windows", arch_name, "zip")
    elif os == "macos":
        return ("macOS", arch_name, "zip")
    elif os == "linux":
        return ("linux", arch_name, "tar.gz")
    return None

# ---------------------------------------------------------------------------
# download_url
# Asset: gh_{version}_{os}_{arch}.{ext}
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _gh_platform(ctx)
    if not platform:
        return None
    os_name, arch_name, ext = platform[0], platform[1], platform[2]
    asset = "gh_{}_{}_{}.{}".format(version, os_name, arch_name, ext)
    return github_asset_url("cli", "cli", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# Windows: bin/gh.exe (no top-level dir)
# macOS:   gh_{version}_macOS_{arch}/bin/gh
# Linux:   gh_{version}_linux_{arch}/bin/gh
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx.platform.os
    if os == "windows":
        return {
            "type":             "archive",
            "strip_prefix":     "",
            "executable_paths": ["bin/gh.exe", "gh.exe"],
        }
    platform = _gh_platform(ctx)
    if not platform:
        return {"type": "archive", "strip_prefix": "", "executable_paths": ["bin/gh"]}
    os_name, arch_name, _ext = platform[0], platform[1], platform[2]
    return {
        "type":             "archive",
        "strip_prefix":     "gh_{}_{}_{}/".format(version, os_name, arch_name),
        "executable_paths": ["bin/gh"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/gh"

def get_execute_path(ctx, _version):
    return ctx.install_dir + ("/gh.exe" if ctx.platform.os == "windows" else "/gh")

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
