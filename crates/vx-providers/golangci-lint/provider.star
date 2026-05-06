# provider.star - golangci-lint provider
#
# golangci-lint: Fast linters runner for Go.
# Releases: https://github.com/golangci/golangci-lint/releases
#
# Asset format (standard goreleaser style, lowercase os/arch, amd64):
#   golangci-lint-{version}-linux-amd64.tar.gz
#   golangci-lint-{version}-linux-arm64.tar.gz
#   golangci-lint-{version}-darwin-amd64.tar.gz
#   golangci-lint-{version}-darwin-arm64.tar.gz
#   golangci-lint-{version}-windows-amd64.zip
#   golangci-lint-{version}-windows-arm64.zip
#
# Archive contains a top-level dir: golangci-lint-{version}-{os}-{arch}/
# Tag format: v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "github_go_provider")
load("@vx//stdlib:github.star", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "golangci-lint"
description = "golangci-lint - Fast linters runner for Go"
homepage    = "https://golangci-lint.run"
repository  = "https://github.com/golangci/golangci-lint"
license     = "GPL-3.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("golangci-lint",
        aliases         = ["golangci"],
        version_pattern = "golangci-lint has version \\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Provider template - github_go_provider
#
# Asset: golangci-lint-{version}-{os}-{arch}.{ext}
# Archive strip prefix: golangci-lint-{version}-{os}-{arch}/
# Repo:  golangci/golangci-lint
# Tag:   v{version}
# ---------------------------------------------------------------------------

_p = github_go_provider(
    "golangci", "golangci-lint",
    asset        = "golangci-lint-{version}-{os}-{arch}.{ext}",
    executable   = "golangci-lint",
    store        = "golangci-lint",
    strip_prefix = "golangci-lint-{version}-{os}-{arch}/",
)

fetch_versions   = make_fetch_versions("vx-org", "mirrors", tag_prefix = "golangci-lint-")
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]
