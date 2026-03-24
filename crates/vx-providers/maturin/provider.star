# provider.star - maturin provider
#
# maturin: Build and publish crates with pyo3, cffi and uniffi bindings
# as well as Rust binaries as Python packages.
# Releases: https://github.com/PyO3/maturin/releases
# Asset format: maturin-{triple}.{ext}  (no version in filename)
# Tag format:   v{version}
#
# Uses Rust target triples for platform naming.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "archive_layout", "path_fns")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "maturin"
description = "maturin - Build and publish crates with pyo3, cffi and uniffi bindings"
homepage    = "https://www.maturin.rs"
repository  = "https://github.com/PyO3/maturin"
license     = "MIT OR Apache-2.0"
ecosystem   = "python"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("maturin",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "maturin \\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — maturin uses standard "v{version}" tags
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("PyO3", "maturin")

# ---------------------------------------------------------------------------
# Platform helpers
# maturin uses Rust target triples, musl for Linux
# ---------------------------------------------------------------------------

_MATURIN_TRIPLES = {
    "windows/x64":   "x86_64-pc-windows-msvc",
    "windows/arm64": "aarch64-pc-windows-msvc",
    "macos/x64":     "x86_64-apple-darwin",
    "macos/arm64":   "aarch64-apple-darwin",
    "linux/x64":     "x86_64-unknown-linux-musl",
    "linux/arm64":   "aarch64-unknown-linux-musl",
}

def _maturin_triple(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _MATURIN_TRIPLES.get(key)

# ---------------------------------------------------------------------------
# download_url — maturin-{triple}.{ext}, tag = "v{version}"
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    triple = _maturin_triple(ctx)
    if not triple:
        return None
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    asset = "maturin-{}.{}".format(triple, ext)
    return github_asset_url("PyO3", "maturin", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — archive contains maturin binary at root
# ---------------------------------------------------------------------------

install_layout = archive_layout("maturin")

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

paths            = path_fns("maturin")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
