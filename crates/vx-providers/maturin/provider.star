# provider.star - maturin provider
#
# maturin: Build and publish crates with pyo3, cffi and uniffi bindings
# as well as Rust binaries as Python packages.
# Releases: https://github.com/PyO3/maturin/releases
# Asset format: maturin-{triple}.{ext}  (no version in filename)
# Tag format:   v{version}
# Linux uses musl for static linking.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "github_rust_provider")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "maturin"
description = "maturin - Build and publish crates with pyo3, cffi and uniffi bindings"
homepage    = "https://www.maturin.rs"
repository  = "https://github.com/PyO3/maturin"
license     = "MIT OR Apache-2.0"
ecosystem   = "python"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("maturin",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "maturin \\d+"},
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
# Asset: maturin-{triple}.{ext}  (no version in filename)
# Tag:   v{version}
# Linux: musl (static binary)
# ---------------------------------------------------------------------------

_p = github_rust_provider(
    "PyO3", "maturin",
    asset      = "maturin-{triple}.{ext}",
    linux_libc = "musl",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
