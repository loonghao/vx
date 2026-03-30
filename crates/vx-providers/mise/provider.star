# provider.star - mise (polyglot dev environment manager)
#
# mise: A polyglot tool version manager (asdf/nvm/pyenv replacement)
# Releases: https://github.com/jdx/mise/releases
# Asset format: mise-v{version}-{os}-{arch}.tar.gz  (custom naming)
# Tag format:   v{version}
#
# Uses custom download_url because mise uses its own naming convention.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "env_prepend", "path_fns",
     "fetch_versions_from_github")
load("@vx//stdlib:layout.star", "archive_layout")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "mise"
description = "mise - Polyglot dev environment manager"
homepage    = "https://mise.jdx.dev"
repository  = "https://github.com/jdx/mise"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [runtime_def("mise", aliases=["mise-en-place"],
                         version_pattern="\\d+\\.\\d+")]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Platform mapping
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("windows", "x64"),
    "windows/arm64": ("windows", "arm64"),
    "macos/x64":     ("macos", "x64"),
    "macos/arm64":   ("macos", "arm64"),
    "linux/x64":     ("linux", "x64"),
    "linux/arm64":   ("linux", "arm64"),
}

def _mise_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# Provider functions
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_github("jdx", "mise")

def download_url(ctx, version):
    platform = _mise_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://github.com/jdx/mise/releases/download/v{}/mise-v{}-{}-{}.{}".format(
        version, version, os_str, arch_str, ext)

install_layout = archive_layout("mise",
    strip_prefix = "mise",
)

paths = path_fns("mise")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
