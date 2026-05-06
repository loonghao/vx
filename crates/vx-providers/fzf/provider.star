# provider.star - fzf provider
#
# fzf: A command-line fuzzy finder
# Asset naming: fzf-{version}-{os}_{arch}.{ext}  (Go-style with underscore separator)
#
# Uses github_go_provider template from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "github_go_provider", "runtime_def", "github_permissions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "fzf"
description = "fzf - A command-line fuzzy finder"
homepage    = "https://github.com/junegunn/fzf"
repository  = "https://github.com/junegunn/fzf"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("fzf",
        version_pattern = "^\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Provider template — github_go_provider
#
# Asset: fzf-{version}-{os}_{arch}.{ext}
# Tag:   v{version}
# Note: fzf uses underscore between os and arch (linux_amd64, darwin_arm64)
# The {os} and {arch} placeholders use Go-style values (linux/darwin/windows, amd64/arm64)
# ---------------------------------------------------------------------------

_p = github_go_provider(
    "junegunn", "fzf",
    asset = "fzf-{version}-{os}_{arch}.{ext}",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]
