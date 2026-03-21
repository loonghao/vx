# provider.star - cross (cross-compilation tool for Rust)
#
# cross: Zero setup cross compilation and cross testing for Rust.
# Releases: https://github.com/cross-rs/cross/releases
# Asset format: cross-{triple}.tar.gz  (no version in asset name)
# Tag format:   v{version}
#
# NOTE: cross assets do NOT include the version number in the filename.
# Only x86_64 pre-built binaries are available (no arm64).
# Requires Docker or Podman to be installed on the host.
# License: MIT OR Apache-2.0
# Homepage: https://github.com/cross-rs/cross

load("@vx//stdlib:provider.star",
     "runtime_def", "dep_def",
     "github_permissions",
     "platform_map")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "cross"
description = "cross - Zero setup cross compilation and cross testing for Rust"
homepage    = "https://github.com/cross-rs/cross"
repository  = "https://github.com/cross-rs/cross"
license     = "MIT OR Apache-2.0"
ecosystem   = "rust"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("cross",
        description     = "Cross compilation tool for Rust",
        version_pattern = "cross \\d+\\.\\d+\\.\\d+",
        test_commands   = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "cross \\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("cross-rs", "cross")

# ---------------------------------------------------------------------------
# Platform helpers
#
# cross only provides x86_64 pre-built binaries:
#   cross-x86_64-unknown-linux-gnu.tar.gz
#   cross-x86_64-unknown-linux-musl.tar.gz
#   cross-x86_64-pc-windows-msvc.tar.gz
#   cross-x86_64-apple-darwin.tar.gz
#
# NOTE: Asset filenames do NOT contain the version number.
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "linux/x64":    "x86_64-unknown-linux-musl",
    "windows/x64":  "x86_64-pc-windows-msvc",
    "macos/x64":    "x86_64-apple-darwin",
    "macos/arm64":  "x86_64-apple-darwin",  # Rosetta 2 fallback
}

def download_url(ctx, version):
    triple = platform_map(ctx, _PLATFORMS)
    if not triple:
        return None
    return "https://github.com/cross-rs/cross/releases/download/v{}/cross-{}.tar.gz".format(
        version, triple)

# ---------------------------------------------------------------------------
# install_layout — archive with executables at root
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    if ctx.platform.os == "windows":
        exe_paths = ["cross.exe", "cross-util.exe"]
    else:
        exe_paths = ["cross", "cross-util"]
    return {
        "__type":           "archive",
        "strip_prefix":     None,
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/cross"

def get_execute_path(ctx, _version):
    exe = "cross.exe" if ctx.platform.os == "windows" else "cross"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

# ---------------------------------------------------------------------------
# deps — cross requires Docker or Podman, and Rust toolchain
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("cargo", optional = False,
                reason = "cross delegates to cargo for building Rust projects"),
    ]
