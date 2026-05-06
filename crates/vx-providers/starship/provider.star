load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_rust_provider")
load("@vx//stdlib:system_install.star", "cross_platform_install")
load("@vx//stdlib:github.star", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "starship"
description = "Starship - The minimal, blazing-fast, and infinitely customizable prompt for any shell"
homepage    = "https://starship.rs"
repository  = "https://github.com/starship/starship"
license     = "ISC"
ecosystem   = "custom"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------
runtimes = [
    runtime_def("starship",
        aliases         = ["starship-prompt"],
        version_pattern  = "\\d+\\.\\d+\\.\\d+",
        test_commands   = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+\\.\\d+"},
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
# Starship uses standard Rust target triple naming:
#   starship-x86_64-unknown-linux-musl.tar.gz
#   starship-x86_64-pc-windows-msvc.zip
#   starship-aarch64-apple-darwin.tar.gz
_p = github_rust_provider("starship", "starship",
    asset      = "starship-{triple}.{ext}",
    executable = "starship",
)

fetch_versions   = make_fetch_versions("vx-org", "mirrors", tag_prefix = "starship-")
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]

system_install = cross_platform_install(
    windows = "starship",
    macos   = "starship",
    linux   = "starship",
)
