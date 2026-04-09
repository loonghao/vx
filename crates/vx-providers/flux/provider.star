# provider.star - Flux provider
#
# Flux is a tool for keeping Kubernetes clusters in sync with sources of
# configuration (like Git repositories), and automating updates to that
# configuration when there is new code to deploy.
#
# Release assets use goreleaser format:
#   flux_{version}_{os}_{arch}.{ext}
# e.g. flux_2.8.5_linux_amd64.tar.gz, flux_2.8.5_windows_amd64.zip
#
# Archive contains: flux[.exe] at the root

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "github_go_provider")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "flux"
description = "Flux - GitOps toolkit for keeping Kubernetes clusters in sync"
homepage    = "https://fluxcd.io"
repository  = "https://github.com/fluxcd/flux2"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("flux",
        aliases         = ["flux2"],
        version_cmd     = "{executable} version --client",
        version_pattern = "flux: v",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Provider template - github_go_provider
#
# Asset: flux_{version}_{os}_{arch}.{ext}
# Repo:  fluxcd/flux2
# Tag:   v{version}
# ---------------------------------------------------------------------------

_p = github_go_provider(
    "fluxcd", "flux2",
    asset      = "flux_{version}_{os}_{arch}.{ext}",
    executable = "flux",
    store      = "flux",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]
