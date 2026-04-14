# provider.star - kustomize provider
#
# kustomize lets you customize raw, template-free YAML files for multiple purposes,
# leaving the original YAML untouched and usable as is.
#
# Release assets (GitHub releases):
#   kustomize_v{version}_{os}_{arch}.tar.gz
#
# OS:   linux, darwin, windows
# Arch: amd64, arm64
#
# Note: kustomize uses a non-standard tag prefix "kustomize/v" (e.g. "kustomize/v5.3.0").
# Version source: kubernetes-sigs/kustomize releases on GitHub

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:github.star", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:layout.star", "archive_layout")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "kustomize"
description = "kustomize - Customization of Kubernetes YAML configurations"
homepage    = "https://kustomize.io"
repository  = "https://github.com/kubernetes-sigs/kustomize"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("kustomize",
        version_cmd     = "{executable} version",
        version_pattern = "v\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions - kustomize uses "kustomize/v" tag prefix
# We only want the kustomize tool releases, not kyaml/api releases
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix(
    "kubernetes-sigs", "kustomize", tag_prefix = "kustomize/v"
)

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("windows", "amd64"),
    "windows/arm64": ("windows", "arm64"),
    "macos/x64":     ("darwin",  "amd64"),
    "macos/arm64":   ("darwin",  "arm64"),
    "linux/x64":     ("linux",   "amd64"),
    "linux/arm64":   ("linux",   "arm64"),
}

def _kustomize_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url
# Asset: kustomize_v{version}_{os}_{arch}.tar.gz
# Tag:   kustomize/v{version}
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _kustomize_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    asset = "kustomize_v{}_{}_{}.{}".format(version, os_str, arch_str, ext)
    tag = "kustomize/v" + version
    return github_asset_url("kubernetes-sigs", "kustomize", tag, asset)

# ---------------------------------------------------------------------------
# Layout + path/env functions
# kustomize archives contain a single kustomize[.exe] binary at the root
# ---------------------------------------------------------------------------

install_layout = archive_layout("kustomize")

paths            = path_fns("kustomize")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]


def post_install(_ctx, _version):
    return None


def deps(_ctx, _version):
    return []
