# provider.star - trippy (network path tracer)
#
# trippy: Combines traceroute and ping into a single tool
# Releases: https://github.com/fujiapple852/trippy/releases
# Asset format: trippy-{version}-{triple}.{ext}  (Rust triple, no v prefix)
# Tag format:   {version}  (no v prefix)

load("@vx//stdlib:provider.star",
     "github_rust_provider", "runtime_def", "github_permissions")

name        = "trippy"
description = "trippy - Network path tracer combining traceroute and ping"
homepage    = "https://trippy.cli.rs"
repository  = "https://github.com/fujiapple852/trippy"
license     = "Apache-2.0"
ecosystem   = "devtools"

runtimes = [runtime_def("trip", aliases=["trippy"],
                         version_pattern="trip \\d+")]

permissions = github_permissions()

_p = github_rust_provider(
    "fujiapple852", "trippy",
    asset        = "trippy-{version}-{triple}.{ext}",
    executable   = "trip",
    tag_prefix   = "",
    strip_prefix = "trippy-{version}-{triple}",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]
