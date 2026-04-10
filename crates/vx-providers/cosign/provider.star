# provider.star - cosign provider
#
# cosign: Container signing, verification and storage in an OCI registry.
# Part of the Sigstore project.
# Releases: https://github.com/sigstore/cosign/releases
#
# Asset format (single binary, no archive, no version in filename):
#   cosign-linux-amd64
#   cosign-linux-arm64
#   cosign-darwin-amd64
#   cosign-darwin-arm64
#   cosign-windows-amd64.exe
#
# Tag format: v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "cosign"
description = "cosign - Container signing, verification and storage in an OCI registry"
homepage    = "https://docs.sigstore.dev/cosign/overview"
repository  = "https://github.com/sigstore/cosign"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("cosign",
        version_pattern = "cosign version \\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("sigstore", "cosign")

# ---------------------------------------------------------------------------
# Platform helpers
#
# cosign uses lowercase os/arch with amd64 (standard Go GOOS/GOARCH naming).
# Single binary, no archive.
# Windows binary has .exe suffix; others have no extension.
# ---------------------------------------------------------------------------

_ARCH_MAP = {
    "x64":   "amd64",
    "arm64": "arm64",
}

_OS_MAP = {
    "linux":   "linux",
    "macos":   "darwin",
    "windows": "windows",
}

def _cosign_platform(ctx):
    os_name   = _OS_MAP.get(ctx.platform.os)
    arch_name = _ARCH_MAP.get(ctx.platform.arch)
    if not os_name or not arch_name:
        return None
    return (os_name, arch_name)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _cosign_platform(ctx)
    if not platform:
        return None
    os_name, arch_name = platform[0], platform[1]
    if ctx.platform.os == "windows":
        asset = "cosign-{}-{}.exe".format(os_name, arch_name)
    else:
        asset = "cosign-{}-{}".format(os_name, arch_name)
    return github_asset_url("sigstore", "cosign", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    if ctx.platform.os == "windows":
        return {
            "type":             "binary",
            "executable_name":  "cosign.exe",
        }
    return {
        "type":             "binary",
        "executable_name":  "cosign",
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/cosign"

def get_execute_path(ctx, _version):
    exe = "cosign.exe" if ctx.platform.os == "windows" else "cosign"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
