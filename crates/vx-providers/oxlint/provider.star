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
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:install.star", "set_permissions")
load("@vx//stdlib:github.star", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")
load("@vx//stdlib:platform.star", "exe_suffix")

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
             "expected_output": "Version: \\d+"},
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
# install_layout — archive contains oxlint-{triple} binary (not just "oxlint")
# The tar.gz files contain a single file named "oxlint-{triple}" (e.g.
# "oxlint-x86_64-unknown-linux-gnu").  We must declare the full name in
# executable_paths so the installer can locate and rename it correctly.
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    triple = _oxc_triple(ctx)
    if not triple:
        return None
    # On Windows the zip contains "oxlint-{triple}.exe"
    suffix = exe_suffix(ctx)
    src_name = "oxlint-{}{}".format(triple, suffix)
    dst_name = "oxlint{}".format(suffix)
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [src_name, dst_name],
    }

# ---------------------------------------------------------------------------
# post_extract — set execute permissions on the extracted binary
# The binary name includes the triple, so we build the name dynamically.
# ---------------------------------------------------------------------------

def post_extract(ctx, _version, _install_dir):
    if ctx.platform.os == "windows":
        return []
    triple = _oxc_triple(ctx)
    if not triple:
        return []
    # Set +x on the extracted oxlint-{triple} binary
    return [set_permissions("oxlint-" + triple, "755")]

# ---------------------------------------------------------------------------
# Path queries + environment
# The executable inside the archive is named "oxlint-{triple}" (not "oxlint"),
# so get_execute_path must return the triple-named path.
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/oxlint"

def get_execute_path(ctx, _version):
    triple = _oxc_triple(ctx)
    if not triple:
        return ctx.install_dir + "/oxlint" + exe_suffix(ctx)
    return ctx.install_dir + "/oxlint-{}{}".format(triple, exe_suffix(ctx))

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
