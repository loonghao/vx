# provider.star - cargo-audit provider
#
# cargo-audit audits Cargo.lock files for crates with security vulnerabilities
# reported to the RustSec Advisory Database.
#
# Releases: https://github.com/rustsec/rustsec/releases
# Tag format:   cargo-audit/v{version}  (path-style prefix with slash)
# Asset format: cargo-audit-{triple}-v{version}.tgz
#   e.g. cargo-audit-x86_64-unknown-linux-musl-v0.22.1.tgz

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:layout.star", "fetch_versions_with_tag_prefix")
load("@vx//stdlib:github.star", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "cargo-audit"
description = "cargo-audit - Audit Cargo.lock for security vulnerabilities"
homepage    = "https://rustsec.org"
repository  = "https://github.com/rustsec/rustsec"
license     = "Apache-2.0 OR MIT"
ecosystem   = "rust"

# `vx cargo:audit` routes directly to this provider's pre-compiled binary
# instead of `cargo install audit` (which fails — audit is a library crate).
ecosystem_aliases = [{"ecosystem": "cargo", "package": "audit"}]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("cargo-audit",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "cargo-audit \\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — strip "cargo-audit/v" prefix from tags
# Tags: cargo-audit/v0.22.1  →  version: 0.22.1
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix(
    "rustsec", "rustsec",
    tag_prefix = "cargo-audit/v",
)

# ---------------------------------------------------------------------------
# Platform triple mapping (musl for Linux portability)
# ---------------------------------------------------------------------------

_TRIPLES = {
    "windows/x64":  "x86_64-pc-windows-msvc",
    "macos/x64":    "x86_64-apple-darwin",
    "macos/arm64":  "aarch64-apple-darwin",
    "linux/x64":    "x86_64-unknown-linux-musl",
    "linux/arm64":  "aarch64-unknown-linux-musl",
}

def _triple(ctx):
    return _TRIPLES.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url
# Asset format: cargo-audit-{triple}-v{version}.{ext}
# e.g. cargo-audit-x86_64-unknown-linux-musl-v0.22.1.tgz
# Windows uses .zip, others use .tgz
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    triple = _triple(ctx)
    if not triple:
        return None
    ext = "zip" if ctx.platform.os == "windows" else "tgz"
    fname = "cargo-audit-{}-v{}.{}".format(triple, version, ext)
    tag   = "cargo-audit/v{}".format(version)
    return github_asset_url("rustsec", "rustsec", tag, fname)

# ---------------------------------------------------------------------------
# install_layout — archive with no top-level directory
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "cargo-audit.exe" if ctx.platform.os == "windows" else "cargo-audit"
    return {
        "__type":           "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "cargo-audit"],
    }

# ---------------------------------------------------------------------------
# Path + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/cargo-audit"

def get_execute_path(ctx, _version):
    exe = "cargo-audit.exe" if ctx.platform.os == "windows" else "cargo-audit"
    return ctx.install_dir + "/bin/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def deps(_ctx, _version):
    return []
