# provider.star - cargo-nextest provider
#
# cargo-nextest is a next-generation test runner for Rust projects.
# It provides faster test execution, better output formatting, and
# improved CI integration compared to `cargo test`.
#
# Releases: https://github.com/nextest-rs/nextest/releases
# Tag format:   cargo-nextest-{version}  (prefix without 'v')
# Asset format: cargo-nextest-{version}-{triple}.tar.gz
#   e.g. cargo-nextest-0.9.132-x86_64-unknown-linux-musl.tar.gz

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:layout.star", "fetch_versions_with_tag_prefix")
load("@vx//stdlib:github.star", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "cargo-nextest"
description = "cargo-nextest - A next-generation test runner for Rust"
homepage    = "https://nexte.st"
repository  = "https://github.com/nextest-rs/nextest"
license     = "Apache-2.0 OR MIT"
ecosystem   = "rust"

# `vx cargo:nextest` routes directly to this provider's pre-compiled binary.
ecosystem_aliases = [{"ecosystem": "cargo", "package": "nextest"}]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("cargo-nextest",
        aliases         = ["nextest"],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "cargo-nextest \\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — strip "cargo-nextest-" prefix from tags
# Tags: cargo-nextest-0.9.132  →  version: 0.9.132
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix(
    "nextest-rs", "nextest",
    tag_prefix = "cargo-nextest-",
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
# Asset format: cargo-nextest-{version}-{triple}.{ext}
# e.g. cargo-nextest-0.9.132-x86_64-unknown-linux-musl.tar.gz
# Windows uses .zip, others use .tar.gz
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    triple = _triple(ctx)
    if not triple:
        return None
    ext   = "zip" if ctx.platform.os == "windows" else "tar.gz"
    fname = "cargo-nextest-{}-{}.{}".format(version, triple, ext)
    tag   = "cargo-nextest-{}".format(version)
    return github_asset_url("nextest-rs", "nextest", tag, fname)

# ---------------------------------------------------------------------------
# install_layout — archive with no top-level directory
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "cargo-nextest.exe" if ctx.platform.os == "windows" else "cargo-nextest"
    return {
        "__type":           "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "cargo-nextest"],
    }

# ---------------------------------------------------------------------------
# Path + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/cargo-nextest"

def get_execute_path(ctx, _version):
    exe = "cargo-nextest.exe" if ctx.platform.os == "windows" else "cargo-nextest"
    return ctx.install_dir + "/bin/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def deps(_ctx, _version):
    return []
