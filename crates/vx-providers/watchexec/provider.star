load("@vx//stdlib:system_install.star", "cross_platform_install")
# provider.star - watchexec (file watcher / command runner)
#
# watchexec: Execute commands when watched files change
# Releases: https://github.com/watchexec/watchexec/releases
# Asset format (Windows):       watchexec-{version}-{triple}.zip
# Asset format (Linux/macOS):   watchexec-{version}-{triple}.tar.xz
# Tag format:   v{version}

load("@vx//stdlib:provider.star",
     "github_rust_provider", "runtime_def", "github_permissions")
load("@vx//stdlib:github.star",   "github_asset_url")
load("@vx//stdlib:platform.star", "rust_triple")

name        = "watchexec"
description = "watchexec - Execute commands when watched files change"
homepage    = "https://watchexec.github.io"
repository  = "https://github.com/watchexec/watchexec"
license     = "Apache-2.0"
ecosystem   = "devtools"

runtimes = [runtime_def("watchexec", version_pattern="watchexec \\d+")]

permissions = github_permissions()

# Use the template for everything except download_url, which needs custom
# extension handling: Windows uses .zip, Linux/macOS use .tar.xz.
_p = github_rust_provider(
    "watchexec", "watchexec",
    asset        = "watchexec-{version}-{triple}.tar.xz",
    executable   = "watchexec",
    strip_prefix = "watchexec-{version}-{triple}",
)

fetch_versions   = _p["fetch_versions"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]

def download_url(ctx, version):
    triple = rust_triple(ctx, "musl")
    if not triple:
        return None
    # Windows releases use .zip; Linux and macOS releases use .tar.xz
    ext = "zip" if ctx.platform.os == "windows" else "tar.xz"
    fname = "watchexec-{}-{}.{}".format(version, triple, ext)
    return github_asset_url("watchexec", "watchexec", "v" + version, fname)

system_install = cross_platform_install(
    windows = "watchexec",
    macos   = "watchexec",
    linux   = "watchexec",
)
