# provider.star - Ninja build system provider
#
# Ninja uses short platform names (win/mac/linux/linux-aarch64).
#
# Uses runtime_def + github_permissions from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "ninja"
description = "Ninja - A small build system with a focus on speed"
homepage    = "https://ninja-build.org/"
repository  = "https://github.com/ninja-build/ninja"
license     = "Apache-2.0"
ecosystem   = "devtools"
aliases     = ["ninja-build"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("ninja",
        aliases         = ["ninja-build"],
        version_pattern = "^\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("ninja-build", "ninja")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

def _ninja_platform(ctx):
    os = ctx.platform.os
    arch = ctx.platform.arch
    if os == "windows" and arch == "x64":
        return "win"
    elif os == "macos":
        return "mac"
    elif os == "linux" and arch == "x64":
        return "linux"
    elif os == "linux" and arch == "arm64":
        return "linux-aarch64"
    return None

# ---------------------------------------------------------------------------
# download_url — GitHub releases, asset: ninja-{platform}.zip
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _ninja_platform(ctx)
    if not platform:
        return None
    return github_asset_url("ninja-build", "ninja", "v" + version,
                            "ninja-{}.zip".format(platform))

# ---------------------------------------------------------------------------
# install_layout — zip contains executable at root
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "ninja.exe" if ctx.platform.os == "windows" else "ninja"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "ninja"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/ninja"


def get_execute_path(ctx, _version):
    exe = "ninja.exe" if ctx.platform.os == "windows" else "ninja"
    return ctx.install_dir + "/" + exe


def post_install(_ctx, _version):
    return None


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]


def deps(_ctx, _version):
    return []
