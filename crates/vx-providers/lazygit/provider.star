# provider.star - lazygit (Git TUI)
#
# lazygit: A simple terminal UI for git commands
# Releases: https://github.com/jesseduffield/lazygit/releases
# Asset format: lazygit_{version}_{os}_{arch}.{ext}  (Go-style but uses x86_64 instead of amd64)
# Tag format:   v{version}
#
# Uses custom download_url because lazygit uses x86_64 instead of standard Go amd64.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns",
     "fetch_versions_from_github")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:layout.star", "archive_layout")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "lazygit"
description = "lazygit - A simple terminal UI for git commands"
homepage    = "https://github.com/jesseduffield/lazygit"
repository  = "https://github.com/jesseduffield/lazygit"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [runtime_def("lazygit", version_pattern="commit=")]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Platform mapping
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("Windows", "x86_64"),
    "windows/arm64": ("Windows", "arm64"),
    "macos/x64":     ("Darwin", "x86_64"),
    "macos/arm64":   ("Darwin", "arm64"),
    "linux/x64":     ("Linux", "x86_64"),
    "linux/arm64":   ("Linux", "arm64"),
}

def _lazygit_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# Provider functions
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_github("jesseduffield", "lazygit")

def download_url(ctx, version):
    platform = _lazygit_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://github.com/jesseduffield/lazygit/releases/download/v{}/lazygit_{}_{}_{}.{}".format(
        version, version, os_str, arch_str, ext)

install_layout = archive_layout("lazygit")

paths = path_fns("lazygit")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
