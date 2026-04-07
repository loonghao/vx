# provider.star - tokei provider
#
# tokei: Count your code, quickly.
# Releases: https://github.com/XAMPPRocky/tokei/releases
# Asset format: tokei-{triple}.tar.gz (Unix) / tokei-{triple}.exe (Windows)
# Tag format:   v{version}
#
# NOTE: tokei assets embed the Rust target triple but NOT the version number.
# Windows ships a direct .exe binary; Unix ships a .tar.gz containing the binary.
#
# Supported triples (from v12.1.2 release):
#   windows/x64  → x86_64-pc-windows-msvc (.exe)
#   windows/x86  → i686-pc-windows-msvc   (.exe)
#   macos/x64    → x86_64-apple-darwin    (.tar.gz)
#   linux/x64    → x86_64-unknown-linux-musl (.tar.gz)
#   linux/arm64  → aarch64-unknown-linux-gnu (.tar.gz)
#
# NOTE: tokei has never released a native aarch64-apple-darwin binary.
# macOS arm64 users can run via Rosetta 2 (x86_64 binary works transparently).

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "binary_layout", "archive_layout", "path_fns")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "tokei"
description = "tokei - Count your code, quickly"
homepage    = "https://github.com/XAMPPRocky/tokei"
repository  = "https://github.com/XAMPPRocky/tokei"
license     = "MIT OR Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("tokei",
        version_pattern = "tokei \\d+",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "tokei \\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — standard GitHub releases with v-prefix tags
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("XAMPPRocky", "tokei")

# ---------------------------------------------------------------------------
# Platform → Rust triple mapping
#
# tokei asset names embed the Rust target triple without a version number:
#   Windows: tokei-{triple}.exe      (direct binary)
#   Unix:    tokei-{triple}.tar.gz   (archive)
# ---------------------------------------------------------------------------

_TRIPLES = {
    "windows/x64":  "x86_64-pc-windows-msvc",
    "windows/x86":  "i686-pc-windows-msvc",
    "macos/x64":    "x86_64-apple-darwin",
    # macos/arm64 is intentionally omitted: tokei has never released a native
    # aarch64-apple-darwin binary. macOS arm64 (Apple Silicon) users are
    # expected to run the x86_64 binary via Rosetta 2.
    "linux/x64":    "x86_64-unknown-linux-musl",
    "linux/arm64":  "aarch64-unknown-linux-gnu",
}

def _triple(ctx):
    return _TRIPLES.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    triple = _triple(ctx)
    if not triple:
        return None
    tag = "v" + version
    if ctx.platform.os == "windows":
        asset = "tokei-{}.exe".format(triple)
    else:
        asset = "tokei-{}.tar.gz".format(triple)
    return github_asset_url("XAMPPRocky", "tokei", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
#
# Windows: direct .exe binary → binary_layout
# Unix:    .tar.gz archive    → archive_layout (binary at archive root)
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    if ctx.platform.os == "windows":
        return binary_layout("tokei")(ctx, version)
    return archive_layout("tokei")(ctx, version)

# ---------------------------------------------------------------------------
# Path + environment helpers
# ---------------------------------------------------------------------------

_paths       = path_fns("tokei")
store_root   = _paths["store_root"]

def get_execute_path(ctx, _version):
    exe = "tokei.exe" if ctx.platform.os == "windows" else "tokei"
    return ctx.install_dir + "/bin/" + exe

def environment(ctx, _version):
    return [{"op": "prepend", "name": "PATH", "value": ctx.install_dir + "/bin"}]

def deps(_ctx, _version):
    return []
