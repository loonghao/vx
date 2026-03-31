# provider.star - trivy provider
#
# trivy: A comprehensive security scanner
# Releases: https://github.com/aquasecurity/trivy/releases
# Asset format: trivy_{version}_{OS}-{Arch}.{ext}
# Tag format:   v{version}
#
# Uses custom download_url because trivy uses non-standard naming:
#   - OS is capitalized (Linux, macOS, windows)
#   - Arch uses 64bit/ARM64/32bit instead of amd64/arm64

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:layout.star", "archive_layout")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "trivy"
description = "trivy - A comprehensive security scanner for containers and code"
homepage    = "https://github.com/aquasecurity/trivy"
repository  = "https://github.com/aquasecurity/trivy"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [runtime_def("trivy", version_pattern="Version: \\d+")]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Platform mapping
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("windows", "64bit"),
    "macos/x64":     ("macOS", "64bit"),
    "macos/arm64":   ("macOS", "ARM64"),
    "linux/x64":     ("Linux", "64bit"),
    "linux/arm64":   ("Linux", "ARM64"),
}

# ---------------------------------------------------------------------------
# Provider functions
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix("aquasecurity", "trivy", tag_prefix = "v")

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://github.com/aquasecurity/trivy/releases/download/v{}/trivy_{}_{}-{}.{}".format(
        version, version, os_str, arch_str, ext)

install_layout = archive_layout("trivy")

paths = path_fns("trivy")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
