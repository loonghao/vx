# provider.star - Hadolint provider
#
# Hadolint is a Dockerfile linter that helps you build best practice Docker images.
#
# Inheritance level: 2 (partial override)
#   - fetch_versions: fully inherited from @vx//stdlib:github.star
#   - download_url:   overridden — hadolint uses direct binary downloads (no archive)
#                     with {os}-{arch} naming (not Rust triple), and Windows only
#                     supports x86_64
#
# Release URL format:
#   https://github.com/hadolint/hadolint/releases/download/v{version}/hadolint-{os}-{arch}[.exe]

load("@vx//stdlib:provider.star",
     "github_binary_provider", "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "github_asset_url")
# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "hadolint"
description = "Hadolint - Dockerfile linter, validate inline bash, written in Haskell"
homepage    = "https://github.com/hadolint/hadolint"
repository  = "https://github.com/hadolint/hadolint"
license     = "GPL-3.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("hadolint",
        description     = "Dockerfile linter",
        version_pattern = "Haskell Dockerfile Linter"),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Provider template — binary download, custom OS/arch naming
#
# hadolint uses: hadolint-{OS}-{arch}[.exe]
#   OS:   Linux / Darwin / Windows  (capitalized)
#   arch: x86_64 / arm64
#   Windows only supports x86_64
# ---------------------------------------------------------------------------

_p = github_binary_provider(
    "hadolint", "hadolint",
    asset      = "hadolint{exe}",   # placeholder — overridden below
    executable = "hadolint",
)

store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]
fetch_versions   = _p["fetch_versions"]

# ---------------------------------------------------------------------------
# download_url — custom override
#
# hadolint uses capitalized OS names and x86_64/arm64 arch strings.
# Windows only supports x86_64.
# ---------------------------------------------------------------------------

_OS_MAP = {
    "linux":   "Linux",
    "macos":   "Darwin",
    "windows": "Windows",
}

_ARCH_MAP = {
    "x64":   "x86_64",
    "arm64": "arm64",
}

def download_url(ctx, version):
    os_str   = _OS_MAP.get(ctx.platform.os)
    arch_str = _ARCH_MAP.get(ctx.platform.arch)

    if not os_str or not arch_str:
        return None
    # Windows only supports x86_64
    if ctx.platform.os == "windows" and ctx.platform.arch != "x64":
        return None

    ext   = ".exe" if ctx.platform.os == "windows" else ""
    asset = "hadolint-{}-{}{}".format(os_str, arch_str, ext)
    return github_asset_url("hadolint", "hadolint", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — binary, rename to standard hadolint[.exe]
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    os_str   = _OS_MAP.get(ctx.platform.os, ctx.platform.os)
    arch_str = _ARCH_MAP.get(ctx.platform.arch, ctx.platform.arch)
    ext      = ".exe" if ctx.platform.os == "windows" else ""

    source_name = "hadolint-{}-{}{}".format(os_str, arch_str, ext)
    target_name = "hadolint" + ext

    return {
        "source_name":      source_name,
        "target_name":      target_name,
        "target_dir":       "bin",
        "executable_paths": ["bin/" + target_name],
    }

# ---------------------------------------------------------------------------
# environment (inherited from template)
# ---------------------------------------------------------------------------
# Note: environment is already provided by _p["environment"] above
