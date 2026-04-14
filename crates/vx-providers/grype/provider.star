# provider.star - grype provider
#
# grype: A vulnerability scanner for container images and filesystems.
# Companion tool to syft (both from Anchore).
# Releases: https://github.com/anchore/grype/releases
#
# Asset format (goreleaser standard format):
#   grype_{version}_linux_amd64.tar.gz
#   grype_{version}_linux_arm64.tar.gz
#   grype_{version}_darwin_amd64.tar.gz
#   grype_{version}_darwin_arm64.tar.gz
#   grype_{version}_windows_amd64.zip
#   grype_{version}_windows_arm64.zip
#
# Tag format: v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "github_go_provider")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "grype"
description = "grype - A vulnerability scanner for container images and filesystems"
homepage    = "https://github.com/anchore/grype"
repository  = "https://github.com/anchore/grype"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("grype",
        version_pattern = "grype \\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Provider template - github_go_provider
#
# Asset: grype_{version}_{os}_{arch}.{ext}
# Repo:  anchore/grype
# Tag:   v{version}
# ---------------------------------------------------------------------------

_p = github_go_provider(
    "anchore", "grype",
    asset      = "grype_{version}_{os}_{arch}.{ext}",
    executable = "grype",
    store      = "grype",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]
