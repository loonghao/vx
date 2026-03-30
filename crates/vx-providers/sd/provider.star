# provider.star - sd (intuitive sed alternative)
#
# sd: Intuitive find & replace CLI
# Releases: https://github.com/chmln/sd/releases
# Asset format: sd-v{version}-{triple}.{ext}  (Rust triple)
# Tag format:   v{version}

load("@vx//stdlib:provider.star",
     "github_rust_provider", "runtime_def", "github_permissions")

name        = "sd"
description = "sd - Intuitive find & replace CLI (sed alternative)"
homepage    = "https://github.com/chmln/sd"
repository  = "https://github.com/chmln/sd"
license     = "MIT"
ecosystem   = "devtools"

runtimes = [runtime_def("sd", version_pattern="sd \\d+")]

permissions = github_permissions()

_p = github_rust_provider(
    "chmln", "sd",
    asset        = "sd-{vversion}-{triple}.{ext}",
    executable   = "sd",
    strip_prefix = "sd-{vversion}-{triple}",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]
