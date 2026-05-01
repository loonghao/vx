# provider.star - Protocol Buffers compiler (protoc)
#
# protoc uses non-standard platform naming:
#   win64 / win32 / linux-x86_64 / linux-aarch_64 / osx-universal_binary
# Archive contains bin/protoc[.exe] and include/ headers.
#
# Uses runtime_def + github_permissions from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "protoc"
description = "Protocol Buffers compiler"
homepage    = "https://protobuf.dev"
repository  = "https://github.com/protocolbuffers/protobuf"
license     = "BSD-3-Clause"
ecosystem   = "devtools"
aliases     = ["protobuf"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("protoc",
        aliases         = ["protobuf"],
        version_pattern = "libprotoc \\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["objects.githubusercontent.com"])

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("protocolbuffers", "protobuf")

# ---------------------------------------------------------------------------
# Platform helpers
#
# protoc uses: win64 / win32 / linux-x86_64 / linux-aarch_64 / osx-universal_binary
# ---------------------------------------------------------------------------

_PROTOC_PLATFORMS = {
    "windows/x64":  "win64",
    "windows/x86":  "win32",
    "macos/x64":    "osx-universal_binary",
    "macos/arm64":  "osx-universal_binary",
    "linux/x64":    "linux-x86_64",
    "linux/arm64":  "linux-aarch_64",
    "linux/x86":    "linux-x86_32",
}

def _protoc_platform(ctx):
    return _PROTOC_PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url — GitHub releases, asset: protoc-{version}-{platform}.zip
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _protoc_platform(ctx)
    if not platform:
        return None
    asset = "protoc-{}-{}.zip".format(version, platform)
    return github_asset_url("protocolbuffers", "protobuf", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — archive contains bin/protoc[.exe] + include/
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "bin/protoc.exe" if ctx.platform.os == "windows" else "bin/protoc"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "bin/protoc"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/protoc"

def get_execute_path(ctx, _version):
    exe = "bin/protoc.exe" if ctx.platform.os == "windows" else "bin/protoc"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def deps(_ctx, _version):
    return []

system_install = cross_platform_install(
    windows = "protoc",
    macos   = "protoc",
    linux   = "protoc",
)
