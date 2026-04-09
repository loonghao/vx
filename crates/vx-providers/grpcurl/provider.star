# provider.star - grpcurl provider
#
# grpcurl is a command-line tool that lets you interact with gRPC servers.
# It is basically curl for gRPC servers.
#
# Release assets (GitHub releases, goreleaser format):
#   grpcurl_{version}_{os}_{arch}.tar.gz  (Linux/macOS)
#   grpcurl_{version}_{os}_{arch}.zip     (Windows)
#
# OS:   linux, darwin, windows
# Arch: amd64, arm64
#
# Version source: fullstorydev/grpcurl releases on GitHub (tag prefix "v")

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "github_go_provider")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "grpcurl"
description = "grpcurl - Like curl, but for gRPC: command-line tool for gRPC servers"
homepage    = "https://github.com/fullstorydev/grpcurl"
repository  = "https://github.com/fullstorydev/grpcurl"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("grpcurl",
        version_cmd     = "{executable} version",
        version_pattern = "grpcurl v\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Provider template - github_go_provider
#
# Asset: grpcurl_{version}_{os}_{arch}.{ext}
# Repo:  fullstorydev/grpcurl
# Tag:   v{version}
# ---------------------------------------------------------------------------

_p = github_go_provider(
    "fullstorydev", "grpcurl",
    asset      = "grpcurl_{version}_{os}_{arch}.{ext}",
    executable = "grpcurl",
    store      = "grpcurl",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]
