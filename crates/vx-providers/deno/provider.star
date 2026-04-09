# provider.star - Deno provider
#
# Deno - A modern runtime for JavaScript and TypeScript
# Downloads from GitHub releases (denoland/deno)
#
# Asset format: deno-{triple}.zip  (always zip, Rust triple naming)
# Tag format:   v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "github_rust_provider")
load("@vx//stdlib:env.star", "env_set", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "deno"
description = "Deno - A modern runtime for JavaScript and TypeScript"
homepage    = "https://deno.land"
repository  = "https://github.com/denoland/deno"
license     = "MIT"
ecosystem   = "nodejs"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("deno",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "deno \\d"},
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
# Asset: deno-{triple}.zip  (Deno always uses .zip for all platforms)
# Tag:   v{version}
# ---------------------------------------------------------------------------

_p = github_rust_provider(
    "denoland", "deno",
    asset = "deno-{triple}.zip",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]

def environment(ctx, _version):
    return [
        env_prepend("PATH", ctx.install_dir),
        env_set("DENO_HOME", ctx.install_dir),
    ]
