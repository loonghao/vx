# provider.star - .NET SDK provider
#
# Version source: Microsoft dotnet-releases API
#   https://dotnetcli.blob.core.windows.net/dotnet/release-metadata/releases-index.json
#
# .NET SDK is a large file (~200MB), cross-platform, official Microsoft distribution.
#
# Inheritance pattern: Level 1 (fully custom - uses Microsoft API, not GitHub)

load("@vx//stdlib:env.star", "env_set", "env_prepend")
load("@vx//stdlib:http.star", "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "dotnet"
description = ".NET SDK - Free, cross-platform, open-source developer platform for C#, F#, and VB.NET"
homepage    = "https://dotnet.microsoft.com"
repository  = "https://github.com/dotnet/sdk"
license     = "MIT"
ecosystem   = "devtools"
aliases     = ["dotnet-sdk"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "dotnet",
        "executable":  "dotnet",
        "description": ".NET SDK runtime",
        "aliases":     ["dotnet-sdk"],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "^\\d+\\.\\d+"},
            {"command": "{executable} --info", "name": "info_check", "expected_output": "Runtime Environment"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["dotnetcli.blob.core.windows.net", "dotnetcli.azureedge.net", "builds.dotnet.microsoft.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — Microsoft dotnet-releases API
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch .NET SDK versions from Microsoft's official releases API.

    Uses the dotnet-releases index which provides:
    - All supported .NET versions (LTS and STS)
    - Direct download URLs per platform
    - No rate limiting

    Returns a descriptor dict for the Rust runtime to execute.
    """
    return fetch_json_versions(
        ctx,
        "https://dotnetcli.blob.core.windows.net/dotnet/release-metadata/releases-index.json",
        "dotnet_releases",
    )

# ---------------------------------------------------------------------------
# download_url — Microsoft CDN
# ---------------------------------------------------------------------------

def _dotnet_rid(ctx):
    """Map vx platform to .NET Runtime Identifier (RID)."""
    os   = ctx.platform.os
    arch = ctx.platform.arch

    rids = {
        "windows/x64":   "win-x64",
        "windows/x86":   "win-x86",
        "windows/arm64": "win-arm64",
        "macos/x64":     "osx-x64",
        "macos/arm64":   "osx-arm64",
        "linux/x64":     "linux-x64",
        "linux/arm64":   "linux-arm64",
        "linux/armv7":   "linux-arm",
    }
    return rids.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the .NET SDK download URL from Microsoft CDN.

    Args:
        ctx:     Provider context
        version: .NET SDK version string, e.g. "8.0.100"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    rid = _dotnet_rid(ctx)
    if not rid:
        return None

    os = ctx.platform.os

    # Extract major.minor channel from version (e.g. "8.0.100" -> "8.0")
    parts = version.split(".")
    if len(parts) < 2:
        return None
    channel = parts[0] + "." + parts[1]

    if os == "windows":
        ext = "zip"
    else:
        ext = "tar.gz"

    # Microsoft CDN URL pattern:
    # https://dotnetcli.azureedge.net/dotnet/Sdk/{version}/dotnet-sdk-{version}-{rid}.{ext}
    filename = "dotnet-sdk-{}-{}.{}".format(version, rid, ext)
    return "https://dotnetcli.azureedge.net/dotnet/Sdk/{}/{}".format(version, filename)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    os = ctx.platform.os
    exe = "dotnet.exe" if os == "windows" else "dotnet"
    return {
        "type":             "archive",
        "strip_prefix":     "",   # .NET SDK archives have flat layout
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    return [
        env_set("DOTNET_ROOT", ctx.install_dir),
        env_prepend("PATH", ctx.install_dir),
    ]

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, version):
    """Recommend git and nuget for .NET development."""
    return [
        {"runtime": "git",   "version": "*", "optional": True,
         "reason": "Git is recommended for NuGet package sources"},
        {"runtime": "nuget", "version": "*", "optional": True,
         "reason": "NuGet CLI for advanced package management"},
    ]


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/dotnet"

def get_execute_path(ctx, _version):
    os = ctx.platform.os
    if os == "windows":
        return ctx.install_dir + "/dotnet.exe"
    else:
        return ctx.install_dir + "/dotnet"

def post_install(_ctx, _version):
    return None
