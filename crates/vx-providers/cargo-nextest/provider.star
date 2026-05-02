load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:layout.star", "archive_layout", "path_fns", "path_env_fns")
load("@vx//stdlib:system_install.star", "cross_platform_install")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "cargo-nextest"
description = "cargo-nextest - Next-generation test runner for Rust"
homepage    = "https://nexte.st/"
repository  = "https://github.com/nexte-st-rs/nexte-st"
license     = "Apache-2.0 OR MIT"
ecosystem   = "rust"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------
runtimes = [
    runtime_def("cargo-nextest",
        aliases         = ["nextest"],
        version_pattern  = "\\d+\\.\\d+\\.\\d+",
        test_commands   = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "cargo-nextest \\d+\\.\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------
permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions: tags are "cargo-nextest-0.9.133" (include tool name)
# ---------------------------------------------------------------------------
fetch_versions = make_fetch_versions("nextest-rs", "nextest", tag_prefix="cargo-nextest-")

# ---------------------------------------------------------------------------
# download_url: asset naming: cargo-nextest-{version}-{triple}.tar.gz
#   version = "0.9.133" (tag_prefix stripped by fetch_versions)
# ---------------------------------------------------------------------------
def download_url(ctx, version):
    triple = ctx.platform.rust_triple
    if not triple:
        return None
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    asset = "cargo-nextest-{}-{}.{}".format(version, triple, ext)
    return github_asset_url("nextest-rs", "nextest", "cargo-nextest-" + version, asset)

# ---------------------------------------------------------------------------
# Layout / paths / env
# ---------------------------------------------------------------------------
_install = archive_layout("cargo-nextest")
install_layout   = _install["install_layout"]
store_root       = _install["store_root"]
get_execute_path = _install["get_execute_path"]

_env_fns   = path_env_fns()
environment = _env_fns["environment"]

# system_install fallback when GitHub download is unavailable
system_install = cross_platform_install(
    windows = "cargo-nextest",
    macos   = "cargo-nextest",
    linux   = "cargo-nextest",
)

def deps(_ctx, _version):
    return []
