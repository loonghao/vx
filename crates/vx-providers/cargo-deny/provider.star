load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_rust_provider")
load("@vx//stdlib:system_install.star", "cross_platform_install")
load("@vx//stdlib:github.star", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "cargo-deny"
description = "cargo-deny - Cargo plugin for auditing dependencies"
homepage    = "https://embarkstudios.github.io/cargo-deny/"
repository  = "https://github.com/EmbarkStudios/cargo-deny"
license     = "MIT OR Apache-2.0"
ecosystem   = "rust"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------
runtimes = [
    runtime_def("cargo-deny",
        aliases         = ["deny"],
        version_pattern  = "\\d+\\.\\d+\\.\\d+",
        test_commands   = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "cargo-deny \\d+\\.\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------
permissions = github_permissions()

# ---------------------------------------------------------------------------
# Use github_rust_provider template
# ---------------------------------------------------------------------------
# cargo-deny assets are ALL .tar.gz (including Windows):
#   cargo-deny-0.19.4-x86_64-unknown-linux-gnu.tar.gz
#   cargo-deny-0.19.4-aarch64-apple-darwin.tar.gz
#   cargo-deny-0.19.4-x86_64-pc-windows-msvc.tar.gz
_p = github_rust_provider("EmbarkStudios", "cargo-deny",
    asset      = "cargo-deny-{version}-{triple}.tar.gz",
    executable = "cargo-deny",
    tag_prefix = "",
)

fetch_versions   = make_fetch_versions("vx-org", "mirrors", tag_prefix = "cargo-deny-")
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]

# system_install fallback when GitHub download is unavailable
system_install = cross_platform_install(
    windows = "cargo-deny",
    macos   = "cargo-deny",
    linux   = "cargo-deny",
)
