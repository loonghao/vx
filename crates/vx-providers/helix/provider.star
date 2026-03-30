# provider.star - helix provider
#
# helix: A post-modern modal text editor with built-in LSP support
# Releases: https://github.com/helix-editor/helix/releases
# Asset format: helix-{version}-{arch}-{os}.tar.xz (Linux/macOS), .zip (Windows)
# Tag format:   {version}  (NO 'v' prefix, CalVer: 25.07.1)
# Binary name:  hx
#
# Uses custom download_url because helix has non-standard naming:
#   - No 'v' prefix in tags
#   - Uses {arch}-{os} ordering instead of Rust triples
#   - Uses tar.xz instead of tar.gz on Unix
#   - Bundled with runtime/grammars in the archive

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "env_prepend", "path_fns",
     "fetch_versions_from_github")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "helix"
description = "helix - A post-modern modal text editor with built-in LSP support"
homepage    = "https://helix-editor.com"
repository  = "https://github.com/helix-editor/helix"
license     = "MPL-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [runtime_def("hx", aliases=["helix"],
                         version_pattern="helix \\d+")]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Platform mapping: (arch, os, ext)
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("x86_64", "windows", "zip"),
    "macos/x64":     ("x86_64", "macos", "tar.xz"),
    "macos/arm64":   ("aarch64", "macos", "tar.xz"),
    "linux/x64":     ("x86_64", "linux", "tar.xz"),
    "linux/arm64":   ("aarch64", "linux", "tar.xz"),
}

# ---------------------------------------------------------------------------
# Provider functions
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_github("helix-editor", "helix")

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return None
    arch, os_str, ext = platform
    return "https://github.com/helix-editor/helix/releases/download/{}/helix-{}-{}-{}.{}".format(
        version, version, arch, os_str, ext)

def install_layout(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return {"__type": "archive", "executable_paths": []}
    arch, os_str, _ext = platform
    exe = "hx.exe" if ctx.platform.os == "windows" else "hx"
    return {
        "__type": "archive",
        "strip_prefix": "helix-{}-{}-{}".format(version, arch, os_str),
        "executable_paths": [exe],
    }

paths = path_fns("helix")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [
        env_prepend("PATH", ctx.install_dir),
        env_prepend("HELIX_RUNTIME", ctx.install_dir + "/runtime"),
    ]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
