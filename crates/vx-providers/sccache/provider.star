load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_rust_provider")
load("@vx//stdlib:system_install.star", "cross_platform_install")
load("@vx//stdlib:github.star", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "sccache"
description = "sccache - Shared Compilation Cache for Rust (and more)"
homepage    = "https://crates.io/crates/sccache"
repository  = "https://github.com/mozilla/sccache"
license     = "Apache-2.0"
ecosystem   = "rust"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------
runtimes = [
    runtime_def("sccache"),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------
permissions = github_permissions()

# ---------------------------------------------------------------------------
# Use github_rust_provider template
# Asset naming: sccache-v0.15.0-x86_64-unknown-linux-musl.tar.gz
# ---------------------------------------------------------------------------
_p = github_rust_provider("mozilla", "sccache",
    asset      = "sccache-{vversion}-{triple}.{ext}",
    executable = "sccache",
)

fetch_versions   = make_fetch_versions("vx-org", "mirrors", tag_prefix = "sccache-")
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]

# ---------------------------------------------------------------------------
# Dependencies
# ---------------------------------------------------------------------------
def deps(_ctx, _version):
    return []

system_install = cross_platform_install(
    windows = "sccache",
    macos   = "sccache",
    linux   = "sccache",
)
