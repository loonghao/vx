# provider.star - buf provider
#
# buf: The Buf CLI for working with Protocol Buffers (Protobuf).
# Releases: https://github.com/bufbuild/buf/releases
#
# Asset format (capitalised OS, x86_64 arch, tarball):
#   buf-Linux-x86_64.tar.gz
#   buf-Linux-arm64.tar.gz
#   buf-Darwin-x86_64.tar.gz
#   buf-Darwin-arm64.tar.gz
#   buf-Windows-x86_64.zip
#   buf-Windows-arm64.zip
#
# Tag format: v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "buf"
description = "buf - The best way to work with Protocol Buffers"
homepage    = "https://buf.build"
repository  = "https://github.com/bufbuild/buf"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("buf",
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

fetch_versions = make_fetch_versions("bufbuild", "buf")

# ---------------------------------------------------------------------------
# Platform helpers
#
# buf uses capitalised OS names (Linux, Darwin, Windows)
# and x86_64 (not amd64) for 64-bit x86.  No version in filename.
# ---------------------------------------------------------------------------

_ARCH_MAP = {
    "x64":   "x86_64",
    "arm64": "arm64",
}

_OS_MAP = {
    "linux":   "Linux",
    "macos":   "Darwin",
    "windows": "Windows",
}

def _buf_platform(ctx):
    os_name   = _OS_MAP.get(ctx.platform.os)
    arch_name = _ARCH_MAP.get(ctx.platform.arch)
    if not os_name or not arch_name:
        return None
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return (os_name, arch_name, ext)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _buf_platform(ctx)
    if not platform:
        return None
    os_name, arch_name, ext = platform[0], platform[1], platform[2]
    asset = "buf-{}-{}.{}".format(os_name, arch_name, ext)
    return github_asset_url("bufbuild", "buf", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "buf.exe" if ctx.platform.os == "windows" else "buf"
    # buf archives contain a top-level "buf/" directory with bin/ inside
    return {
        "type":             "archive",
        "strip_prefix":     "buf",
        "executable_paths": ["bin/" + exe],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/buf"

def get_execute_path(ctx, _version):
    exe = "buf.exe" if ctx.platform.os == "windows" else "buf"
    return ctx.install_dir + "/bin/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def deps(_ctx, _version):
    return []

system_install = cross_platform_install(
    windows = "buf",
    macos   = "buf",
    linux   = "buf",
)
