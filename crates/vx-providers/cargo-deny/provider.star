# provider.star - cargo-deny provider
#
# cargo-deny is a cargo plugin for linting your dependencies.
# It can check for security advisories, license compatibility,
# ban certain crates, and check for duplicate dependencies.
#
# Releases: https://github.com/EmbarkStudios/cargo-deny/releases
# Asset format: cargo-deny-{version}-{triple}.{ext}
# Tag format:   {version}  (NO 'v' prefix)
# Linux uses musl for static linking.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "github_rust_provider")

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
# Provider template - github_rust_provider
#
# Asset format: cargo-deny-{version}-{triple}.{ext}
# Tag format:   {version}  (no v prefix, so tag_prefix = "")
# Linux:        musl (static binary for portability)
# ---------------------------------------------------------------------------

_p = github_rust_provider(
    "EmbarkStudios", "cargo-deny",
    asset      = "cargo-deny-{version}-{triple}.{ext}",
    executable = "cargo-deny",
    store      = "cargo-deny",
    tag_prefix = "",
    linux_libc = "musl",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]