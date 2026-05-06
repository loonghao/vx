load("@vx//stdlib:system_install.star", "cross_platform_install")
load("@vx//stdlib:github.star", "make_fetch_versions")
# provider.star - uv provider
#
# uv: An extremely fast Python package installer and resolver
# Tags have NO 'v' prefix (e.g. "0.10.5"). Asset: uv-{triple}.{ext}
# Bundled runtime: uvx
#
# Uses github_rust_provider template from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "github_rust_provider", "runtime_def", "bundled_runtime_def",
     "github_permissions", "pre_run_ensure_deps")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "uv"
description = "An extremely fast Python package installer and resolver"
homepage    = "https://github.com/astral-sh/uv"
repository  = "https://github.com/astral-sh/uv"
license     = "MIT OR Apache-2.0"
ecosystem   = "python"

# Supported package prefixes for ecosystem:package syntax (RFC 0027)
# Enables `vx uv:<package>` and `vx uvx:<package>` for Python package execution
package_prefixes = ["uv", "uvx"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("uv",
        version_pattern = "uv \\d+\\.\\d+",
    ),
    bundled_runtime_def(
        "uvx",
        bundled_with   = "uv",
        executable     = "uv",
        command_prefix = ["tool", "run"],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Provider template — github_rust_provider
#
# uv tags have NO 'v' prefix: "0.10.5" not "v0.10.5"
# Asset: uv-{triple}.{ext}  (no version in asset name)
# ---------------------------------------------------------------------------

_p = github_rust_provider(
    "astral-sh", "uv",
    asset        = "uv-{triple}.{ext}",
    tag_prefix   = "",
    strip_prefix = "uv-{triple}",
)

fetch_versions   = make_fetch_versions("vx-org", "mirrors", tag_prefix = "uv-")
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]

# ---------------------------------------------------------------------------
# pre_run — ensure .venv before `uv run`
# ---------------------------------------------------------------------------

pre_run = pre_run_ensure_deps("uv",
    trigger_args = ["run"],
    check_file   = "pyproject.toml",
    install_dir  = ".venv",
)

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return []

system_install = cross_platform_install(
    windows = "uv",
    macos   = "uv",
    linux   = "uv",
)
