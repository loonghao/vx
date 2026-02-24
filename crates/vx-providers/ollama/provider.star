# provider.star - Ollama provider
#
# Ollama uses simple platform names (not Rust triples):
#   - darwin          (macOS universal binary)
#   - linux-amd64     linux-arm64
#   - windows-amd64   windows-arm64
#
# Archive format: .tgz (all platforms except Windows which uses .zip)
#
# fetch_versions: fully inherited from github.star
# download_url:   custom override (non-standard platform naming)

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "ollama"
description = "Ollama - Get up and running with large language models locally"
homepage    = "https://ollama.com"
repository  = "https://github.com/ollama/ollama"
license     = "MIT"
ecosystem   = "ai"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "ollama",
        "executable":  "ollama",
        "description": "Ollama - Run large language models locally",
        "aliases":     [],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "ollama version"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — fully inherited from github.star
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("ollama", "ollama", include_prereleases = False)

# ---------------------------------------------------------------------------
# download_url — custom override
#
# Ollama uses simple platform names instead of Rust triples:
#   - macOS: universal binary "darwin" (both x64 and arm64)
#   - Linux: "linux-amd64" or "linux-arm64"
#   - Windows: "windows-amd64" or "windows-arm64"
#
# Asset: "ollama-{target}.{ext}"
# Tag:   "v{version}"
# ---------------------------------------------------------------------------

def _ollama_target(ctx):
    """Map platform to ollama's target string."""
    os   = ctx.platform.os
    arch = ctx.platform.arch

    if os == "macos":
        # macOS uses a universal binary regardless of arch
        return "darwin"
    elif os == "linux":
        if arch == "x64":
            return "linux-amd64"
        elif arch == "arm64":
            return "linux-arm64"
    elif os == "windows":
        if arch == "x64":
            return "windows-amd64"
        elif arch == "arm64":
            return "windows-arm64"

    return None

def download_url(ctx, version):
    """Build the ollama download URL for the given version and platform.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "0.13.5"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    target = _ollama_target(ctx)
    if not target:
        return None

    os  = ctx.platform.os
    ext = "zip" if os == "windows" else "tgz"

    # Asset: "ollama-darwin.tgz", "ollama-linux-amd64.tgz", "ollama-windows-amd64.zip"
    asset = "ollama-{}.{}".format(target, ext)
    tag   = "v{}".format(version)

    return github_asset_url("ollama", "ollama", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    """Describe how to extract the downloaded archive."""
    os  = ctx.platform.os
    exe = "ollama.exe" if os == "windows" else "ollama"

    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "ollama"],
    }

# ---------------------------------------------------------------------------
# store_root — vx-managed install directory
# ---------------------------------------------------------------------------

def store_root(ctx, version):
    """Return the vx store root for this ollama version."""
    return ctx.paths.store_dir + "/ollama/" + version

# ---------------------------------------------------------------------------
# get_execute_path — resolve ollama executable
# ---------------------------------------------------------------------------

def get_execute_path(ctx, _version, install_dir):
    """Return the path to the ollama executable."""
    os  = ctx.platform.os
    exe = "ollama.exe" if os == "windows" else "ollama"
    return ctx.install_dir + "/" + exe

# ---------------------------------------------------------------------------
# post_install — set permissions on Unix
# ---------------------------------------------------------------------------

def post_install(ctx, _version):
    """Set execute permissions on Unix."""
    os = ctx.platform.os
    if os == "windows":
        return []
    return [
        {"type": "set_permissions", "path": ctx.install_dir + "/ollama", "mode": "755"},
    ]

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
