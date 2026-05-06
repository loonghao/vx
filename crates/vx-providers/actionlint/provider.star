load("@vx//stdlib:system_install.star", "cross_platform_install")
load("@vx//stdlib:github.star", "make_fetch_versions")
# provider.star - actionlint (GitHub Actions workflow linter)
#
# actionlint: Static checker for GitHub Actions workflow files
# Releases: https://github.com/rhysd/actionlint/releases
# Asset format: actionlint_{version}_{os}_{arch}.{ext}  (Go-style naming)
# Tag format:   v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "github_go_provider")

name        = "actionlint"
description = "actionlint - Static checker for GitHub Actions workflow files"
homepage    = "https://github.com/rhysd/actionlint"
repository  = "https://github.com/rhysd/actionlint"
license     = "MIT"
ecosystem   = "devtools"

runtimes = [runtime_def("actionlint", version_pattern = "\\d+\\.\\d+\\.\\d+")]

permissions = github_permissions()

_p = github_go_provider(
    "rhysd", "actionlint",
    asset = "actionlint_{version}_{os}_{arch}.{ext}",
)

fetch_versions   = make_fetch_versions("vx-org", "mirrors", tag_prefix = "actionlint-")
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]

system_install = cross_platform_install(
    windows = "actionlint",
    macos   = "actionlint",
    linux   = "actionlint",
)
