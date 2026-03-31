# provider.star - gitleaks (git secret scanner)
#
# gitleaks: Detect and prevent hardcoded secrets in git repos
# Releases: https://github.com/gitleaks/gitleaks/releases
# Asset format: gitleaks_{version}_{os}_{arch}.{ext}
# Tag format:   v{version}
#
# Uses custom download_url because gitleaks uses custom os/arch naming.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns",
     "fetch_versions_from_github")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:layout.star", "archive_layout")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "gitleaks"
description = "gitleaks - Detect and prevent hardcoded secrets in git repos"
homepage    = "https://gitleaks.io"
repository  = "https://github.com/gitleaks/gitleaks"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [runtime_def("gitleaks", version_pattern="v\\d+")]

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
    "macos/x64":     ("darwin", "x64"),
    "macos/arm64":   ("darwin", "arm64"),
    "linux/x64":     ("linux", "x64"),
    "linux/arm64":   ("linux", "arm64"),
}

# ---------------------------------------------------------------------------
# Provider functions
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_github("gitleaks", "gitleaks")

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://github.com/gitleaks/gitleaks/releases/download/v{}/gitleaks_{}_{}_{}.{}".format(
        version, version, os_str, arch_str, ext)

install_layout = archive_layout("gitleaks")

paths = path_fns("gitleaks")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
