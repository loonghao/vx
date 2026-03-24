# provider.star - Google Workspace CLI provider
#
# gws: Google Workspace CLI — one command-line tool for Drive, Gmail,
#       Calendar, Sheets, Docs, Chat, Admin, and more.
# Releases: https://github.com/googleworkspace/cli/releases
# Asset format: gws-{triple}.{ext}
#
# Dynamically built from Google Discovery Service. Includes AI agent skills.

load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_rust_provider")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "gws"
description = "Google Workspace CLI - Drive, Gmail, Calendar, Sheets, Docs, Chat, Admin and more"
homepage    = "https://github.com/googleworkspace/cli"
repository  = "https://github.com/googleworkspace/cli"
license     = "Apache-2.0"
ecosystem   = "custom"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("gws",
        aliases = ["google-workspace-cli"],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Template: Rust target triple naming (gws-{triple}.{ext})
# Linux uses gnu by default for this project
# ---------------------------------------------------------------------------

_p = github_rust_provider("googleworkspace", "cli",
    asset        = "gws-{triple}.{ext}",
    executable   = "gws",
    store        = "gws",
    linux_libc   = "gnu",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]

def deps(_ctx, _version):
    return []
