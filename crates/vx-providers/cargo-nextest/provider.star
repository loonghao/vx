# provider.star - cargo-nextest provider
#
# cargo-nextest is a next-generation test runner for Rust projects.
# It provides faster test execution, better output formatting, and
# improved CI integration compared to `cargo test`.
#
# Releases: https://github.com/nextest-rs/nextest/releases
# Asset format: cargo-nextest-{version}-{triple}.{ext}  (Rust triple naming)
# Tag format:   cargo-nextest-{version}  (custom tag prefix)
# Linux uses musl for static linking.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "github_rust_provider")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "cargo-nextest"
description = "cargo-nextest - A next-generation test runner for Rust"
homepage    = "https://nexte.st"
repository  = "https://github.com/nextest-rs/nextest"
license     = "Apache-2.0 OR MIT"
ecosystem   = "rust"

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
# Provider template - github_rust_provider
#
# Asset format: cargo-nextest-{version}-{triple}.{ext}
# Tag format:   cargo-nextest-{version}  (tag_prefix = "cargo-nextest-")
# Linux:        musl (static binary for portability)
# ---------------------------------------------------------------------------

_p = github_rust_provider(
    "nextest-rs", "nextest",
    asset      = "cargo-nextest-{version}-{triple}.{ext}",
    executable = "cargo-nextest",
    store      = "cargo-nextest",
    tag_prefix = "cargo-nextest-",
    linux_libc = "musl",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]