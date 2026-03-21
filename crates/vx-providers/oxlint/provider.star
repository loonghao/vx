# provider.star - oxlint provider
#
# oxlint: The JavaScript/TypeScript linter from the oxc project
# Releases: https://github.com/oxc-project/oxc/releases
# Asset format: oxlint-{triple}.{ext}  (no version in filename)
# Tag format:   apps_v{version}
#
# Also provides oxfmt (bundled formatter from the same release)

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "github_permissions",
     "archive_layout", "path_fns",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:github.star", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "oxlint"
description = "oxlint - The JavaScript/TypeScript linter powered by Rust (oxc project)"
homepage    = "https://oxc.rs"
repository  = "https://github.com/oxc-project/oxc"
license     = "MIT"
ecosystem   = "nodejs"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("oxlint",
        aliases = ["oxc-lint"],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "oxlint \\d+"},
        ],
    ),
    bundled_runtime_def("oxfmt", bundled_with = "oxlint",
        description = "oxfmt - The JavaScript/TypeScript formatter from the oxc project",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — oxc uses "apps_v{version}" tag format
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix(
    "oxc-project", "oxc", tag_prefix = "apps_v"
)

# ---------------------------------------------------------------------------
# Platform helpers
# oxc uses Rust target triples
# ---------------------------------------------------------------------------

_OXC_TRIPLES = {
    "windows/x64":   "x86_64-pc-windows-msvc",
    "windows/arm64": "aarch64-pc-windows-msvc",
    "macos/x64":     "x86_64-apple-darwin",
    "macos/arm64":   "aarch64-apple-darwin",
    "linux/x64":     "x86_64-unknown-linux-gnu",
    "linux/arm64":   "aarch64-unknown-linux-gnu",
}

def _oxc_triple(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _OXC_TRIPLES.get(key)

# ---------------------------------------------------------------------------
# download_url — oxlint-{triple}.{ext}, tag = "apps_v{version}"
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    triple = _oxc_triple(ctx)
    if not triple:
        return None
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    asset = "oxlint-{}.{}".format(triple, ext)
    return github_asset_url("oxc-project", "oxc", "apps_v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — archive contains oxlint (+ oxfmt) binary at root
# ---------------------------------------------------------------------------

install_layout = archive_layout("oxlint")

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

paths            = path_fns("oxlint")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
