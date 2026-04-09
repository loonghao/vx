# provider.star - cargo-deny provider
#
# cargo-deny is a cargo plugin for linting your dependencies.
#
# Releases: https://github.com/EmbarkStudios/cargo-deny/releases
# Tag format:   {version}  (NO 'v' prefix — tag_prefix = "")
# Asset format: cargo-deny-{version}-{triple}.tar.gz  (always .tar.gz, no .zip)
# Linux uses musl for static linking.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:platform.star", "rust_triple")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "cargo-deny"
description = "cargo-deny - Cargo plugin for linting dependencies"
homepage    = "https://embarkstudios.github.io/cargo-deny/"
repository  = "https://github.com/EmbarkStudios/cargo-deny"
license     = "Apache-2.0 OR MIT"
ecosystem   = "rust"

# `vx cargo:deny` routes to this provider's pre-compiled binary.
ecosystem_aliases = [{"ecosystem": "cargo", "package": "deny"}]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("cargo-deny",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "cargo-deny \\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — tags have NO 'v' prefix (e.g. "0.19.0")
# Standard make_fetch_versions returns full tag name as version, which is
# correct here since tag = version (no prefix to strip).
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("EmbarkStudios", "cargo-deny")

# ---------------------------------------------------------------------------
# download_url
# Asset format: cargo-deny-{version}-{triple}.tar.gz  (always .tar.gz)
# Tag format:   {version}  (no prefix)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    triple = rust_triple(ctx, "musl")
    if not triple:
        return None
    fname = "cargo-deny-{}-{}.tar.gz".format(version, triple)
    return github_asset_url("EmbarkStudios", "cargo-deny", version, fname)

# ---------------------------------------------------------------------------
# install_layout — archive with no top-level directory
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "cargo-deny.exe" if ctx.platform.os == "windows" else "cargo-deny"
    return {
        "__type":           "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "cargo-deny"],
    }

# ---------------------------------------------------------------------------
# Path + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/cargo-deny"

def get_execute_path(ctx, _version):
    exe = "cargo-deny.exe" if ctx.platform.os == "windows" else "cargo-deny"
    return ctx.install_dir + "/bin/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def deps(_ctx, _version):
    return []
