load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_rust_provider")
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
# Use github_rust_provider template
# ---------------------------------------------------------------------------
# nextest tags: "cargo-nextest-0.9.133" (no "v" prefix; includes tool name)
# asset naming: cargo-nextest-0.9.133-x86_64-unknown-linux-gnu.tar.gz
_p = github_rust_provider("nextest-rs", "nextest",
    asset      = "cargo-nextest-{version}-{triple}.{ext}",
    executable = "cargo-nextest",
    tag_prefix = "cargo-nextest-",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]

# system_install fallback when GitHub download is unavailable
system_install = cross_platform_install(
    windows = "cargo-nextest",
    macos   = "cargo-nextest",
    linux   = "cargo-nextest",
)
