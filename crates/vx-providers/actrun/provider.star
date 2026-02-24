# provider.star - actrun provider
#
# actrun: Actionforge workflow runner CLI
# Asset: actrun-v{version}.cli-{arch}-{os}.{ext}
# macOS uses .pkg (not supported) — Windows/Linux only
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "actrun"
description = "Actionforge workflow runner CLI for executing GitHub Actions-compatible workflows locally"
homepage    = "https://github.com/actionforge/actrun-cli"
repository  = "https://github.com/actionforge/actrun-cli"
license     = "Actionforge-EULA"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("actrun",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("actionforge", "actrun-cli")

# ---------------------------------------------------------------------------
# Platform helpers
# Asset: actrun-v{version}.cli-{arch}-{os}.{ext}
# macOS uses .pkg — not supported
# ---------------------------------------------------------------------------

_ACTRUN_PLATFORMS = {
    "windows/x64":   ("x64",   "windows", "zip"),
    "windows/arm64": ("arm64", "windows", "zip"),
    "linux/x64":     ("x64",   "linux",   "tar.gz"),
    "linux/arm64":   ("arm64", "linux",   "tar.gz"),
}

def download_url(ctx, version):
    platform = _ACTRUN_PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))
    if not platform:
        return None
    act_arch, act_os, ext = platform[0], platform[1], platform[2]
    asset = "actrun-v{}.cli-{}-{}.{}".format(version, act_arch, act_os, ext)
    return github_asset_url("actionforge", "actrun-cli", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "actrun.exe" if ctx.platform.os == "windows" else "actrun"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "actrun"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/actrun"

def get_execute_path(ctx, _version):
    exe = "actrun.exe" if ctx.platform.os == "windows" else "actrun"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
