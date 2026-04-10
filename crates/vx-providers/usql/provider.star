# provider.star - usql provider
#
# usql is a universal command-line interface for SQL databases.
# Supports PostgreSQL, MySQL, Oracle, SQLite, MSSQL, and many more.
#
# Release assets (GitHub releases, goreleaser format):
#   usql_{version}_{os}_{arch}.tar.gz  (Linux/macOS)
#   usql_{version}_{os}_{arch}.zip     (Windows)
#
# OS:   linux, darwin, windows
# Arch: amd64, arm64
#
# Version source: xo/usql releases on GitHub (tag prefix "v")

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "github_go_provider")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "usql"
description = "usql - Universal command-line interface for SQL databases"
homepage    = "https://github.com/xo/usql"
repository  = "https://github.com/xo/usql"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("usql",
        version_cmd     = "{executable} --version",
        version_pattern = "usql \\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Provider template - github_go_provider
#
# Asset: usql_{version}_{os}_{arch}.{ext}
# Repo:  xo/usql
# Tag:   v{version}
# ---------------------------------------------------------------------------

_p = github_go_provider(
    "xo", "usql",
    asset      = "usql_{version}_{os}_{arch}.{ext}",
    executable = "usql",
    store      = "usql",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]
