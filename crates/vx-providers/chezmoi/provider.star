# provider.star - chezmoi provider
#
# chezmoi: Manage your dotfiles across multiple machines
# Releases: https://github.com/twpayne/chezmoi/releases
# Asset format: chezmoi_{version}_{os}_{arch}.{ext}  (Go goreleaser style)
# Tag format:   v{version}
#
# Uses github_go_provider template from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "github_go_provider", "runtime_def", "github_permissions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "chezmoi"
description = "chezmoi - Manage your dotfiles across multiple machines"
homepage    = "https://www.chezmoi.io"
repository  = "https://github.com/twpayne/chezmoi"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("chezmoi",
        version_pattern = "chezmoi version v\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Provider template — github_go_provider
#
# Asset: chezmoi_{version}_{os}_{arch}.{ext}
# Tag:   v{version}
# ---------------------------------------------------------------------------

_p = github_go_provider(
    "twpayne", "chezmoi",
    asset = "chezmoi_{version}_{os}_{arch}.{ext}",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]

system_install = cross_platform_install(
    windows = "chezmoi",
    macos   = "chezmoi",
    linux   = "chezmoi",
)
