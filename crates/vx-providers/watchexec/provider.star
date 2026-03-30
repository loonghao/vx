# provider.star - watchexec (file watcher / command runner)
#
# watchexec: Execute commands when watched files change
# Releases: https://github.com/watchexec/watchexec/releases
# Asset format: watchexec-{version}-{triple}.tar.xz  (Rust triple, tar.xz format)
# Tag format:   v{version}

load("@vx//stdlib:provider.star",
     "github_rust_provider", "runtime_def", "github_permissions")

name        = "watchexec"
description = "watchexec - Execute commands when watched files change"
homepage    = "https://watchexec.github.io"
repository  = "https://github.com/watchexec/watchexec"
license     = "Apache-2.0"
ecosystem   = "devtools"

runtimes = [runtime_def("watchexec", version_pattern="watchexec \\d+")]

permissions = github_permissions()

_p = github_rust_provider(
    "watchexec", "watchexec",
    asset        = "watchexec-{version}-{triple}.tar.xz",
    executable   = "watchexec",
    strip_prefix = "watchexec-{version}-{triple}",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]
