# provider.star - lima provider
#
# lima: Linux virtual machines on macOS (and other hosts).
# Releases: https://github.com/lima-vm/lima/releases
#
# Asset format (custom OS/arch names, Pascal-case OS):
#   lima-{version}-Darwin-arm64.tar.gz
#   lima-{version}-Darwin-x86_64.tar.gz
#   lima-{version}-Linux-x86_64.tar.gz
#   lima-{version}-Linux-aarch64.tar.gz
#   lima-{version}-Windows-AMD64.zip
#   lima-{version}-Windows-ARM64.zip
#
# Tag format: v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "lima"
description = "lima - Linux virtual machines, with a focus on running containers"
homepage    = "https://lima-vm.io"
repository  = "https://github.com/lima-vm/lima"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("limactl",
        aliases         = ["lima"],
        version_pattern = "\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("lima-vm", "lima")

# ---------------------------------------------------------------------------
# Platform helpers
#
# lima uses Pascal-case OS names (Darwin/Linux/Windows) and full arch names.
# macOS/Linux: tar.gz; Windows: zip
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "linux/x64":    ("Linux",   "x86_64",  "tar.gz"),
    "linux/arm64":  ("Linux",   "aarch64", "tar.gz"),
    "macos/x64":    ("Darwin",  "x86_64",  "tar.gz"),
    "macos/arm64":  ("Darwin",  "arm64",   "tar.gz"),
    "windows/x64":  ("Windows", "AMD64",   "zip"),
    "windows/arm64":("Windows", "ARM64",   "zip"),
}

def _lima_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _lima_platform(ctx)
    if not platform:
        return None
    os_name, arch_name, ext = platform[0], platform[1], platform[2]
    asset = "lima-{}-{}-{}.{}".format(version, os_name, arch_name, ext)
    return github_asset_url("lima-vm", "lima", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    if ctx.platform.os == "windows":
        return {
            "type":             "archive",
            "strip_prefix":     "",
            "executable_paths": ["bin/limactl.exe"],
        }
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": ["bin/limactl"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/lima"

def get_execute_path(ctx, _version):
    exe = "limactl.exe" if ctx.platform.os == "windows" else "limactl"
    return ctx.install_dir + "/bin/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def deps(_ctx, _version):
    return []
