# provider.star - dive provider
#
# dive: A tool for exploring Docker image layers
# Releases: https://github.com/wagoodman/dive/releases
# Asset format: dive_{version}_{os}_{arch}.{ext}  (Go goreleaser style)
# Tag format:   v{version}
#
# Uses custom download_url because dive uses goreleaser naming
# with lowercase os/arch (linux, darwin, windows, amd64, arm64).

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:layout.star", "archive_layout")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "dive"
description = "dive - A tool for exploring Docker image layers"
homepage    = "https://github.com/wagoodman/dive"
repository  = "https://github.com/wagoodman/dive"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [runtime_def("dive", version_pattern="dive \\d+")]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Platform mapping
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("windows", "amd64"),
    "windows/arm64": ("windows", "arm64"),
    "macos/x64":     ("darwin", "amd64"),
    "macos/arm64":   ("darwin", "arm64"),
    "linux/x64":     ("linux", "amd64"),
    "linux/arm64":   ("linux", "arm64"),
}

# ---------------------------------------------------------------------------
# Provider functions
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix("wagoodman", "dive", tag_prefix = "v")

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://github.com/wagoodman/dive/releases/download/v{}/dive_{}_{}_{}.{}".format(
        version, version, os_str, arch_str, ext)

install_layout = archive_layout("dive")

paths = path_fns("dive")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
