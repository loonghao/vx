# provider.star - syft provider
#
# syft: CLI tool and library for generating a Software Bill of Materials (SBOM)
# from container images and filesystems.
# Releases: https://github.com/anchore/syft/releases
#
# Asset format (goreleaser standard format):
#   syft_{version}_linux_amd64.tar.gz
#   syft_{version}_linux_arm64.tar.gz
#   syft_{version}_darwin_amd64.tar.gz
#   syft_{version}_darwin_arm64.tar.gz
#   syft_{version}_windows_amd64.zip
#   syft_{version}_windows_arm64.zip
#
# Tag format: v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "github_go_provider")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "syft"
description = "syft - CLI tool for generating a Software Bill of Materials (SBOM)"
homepage    = "https://github.com/anchore/syft"
repository  = "https://github.com/anchore/syft"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("syft",
        version_pattern = "syft \\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Provider template - github_go_provider
#
# Asset: syft_{version}_{os}_{arch}.{ext}
# Repo:  anchore/syft
# Tag:   v{version}
# ---------------------------------------------------------------------------

_p = github_go_provider(
    "anchore", "syft",
    asset      = "syft_{version}_{os}_{arch}.{ext}",
    executable = "syft",
    store      = "syft",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]
