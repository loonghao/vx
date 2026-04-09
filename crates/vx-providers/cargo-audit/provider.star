# provider.star - cargo-audit provider
#
# cargo-audit audits Cargo.lock files for crates with security vulnerabilities
# reported to the RustSec Advisory Database.
#
# Releases: https://github.com/rustsec/rustsec/releases
# Asset format: cargo-audit-{version}-{triple}.{ext}
# Tag format:   cargo-audit/v{version}  (uses path-style tag prefix)

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "github_rust_provider")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "cargo-audit"
description = "cargo-audit - Audit Cargo.lock for security vulnerabilities"
homepage    = "https://rustsec.org"
repository  = "https://github.com/rustsec/rustsec"
license     = "Apache-2.0 OR MIT"
ecosystem   = "rust"

# `vx cargo:audit` routes directly to this provider's pre-compiled binary
# instead of `cargo install audit` (which fails — audit is a library crate).
ecosystem_aliases = [{"ecosystem": "cargo", "package": "audit"}]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("cargo-audit",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "cargo-audit \\d+"},
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
# Asset format: cargo-audit-{version}-{triple}.{ext}
# Tag format:   cargo-audit/v{version}  (tag_prefix = "cargo-audit/v")
# Linux:        musl (static binary for portability)
# ---------------------------------------------------------------------------

_p = github_rust_provider(
    "rustsec", "rustsec",
    asset      = "cargo-audit-{version}-{triple}.{ext}",
    executable = "cargo-audit",
    store      = "cargo-audit",
    tag_prefix = "cargo-audit/v",
    linux_libc = "musl",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]