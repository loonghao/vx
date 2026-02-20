# provider.star - .NET SDK provider
#
# Version source: Microsoft dotnet-releases API
#   https://dotnetcli.blob.core.windows.net/dotnet/release-metadata/releases-index.json
#
# .NET SDK is a large file (~200MB), cross-platform, official Microsoft distribution.
#
# Inheritance pattern: Level 1 (fully custom - uses Microsoft API, not GitHub)

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "dotnet"

def description():
    return ".NET SDK - Free, cross-platform, open-source developer platform for C#, F#, and VB.NET"

def homepage():
    return "https://dotnet.microsoft.com"

def repository():
    return "https://github.com/dotnet/sdk"

def license():
    return "MIT"

def ecosystem():
    return "devtools"

def aliases():
    return ["dotnet-sdk"]

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
    """
    index = ctx["http"]["get_json"](
        "https://dotnetcli.blob.core.windows.net/dotnet/release-metadata/releases-index.json"
    )

    versions = []
    for channel in index.get("releases-index", []):
        channel_version = channel.get("channel-version", "")
        support_phase = channel.get("support-phase", "")
        is_lts = channel.get("release-type", "") == "lts"

        # Skip EOL channels
        if support_phase == "eol":
            continue

        # Fetch the releases for this channel
        releases_url = channel.get("releases.json", "")
        if not releases_url:
            continue

        channel_releases = ctx["http"]["get_json"](releases_url)
        for release in channel_releases.get("releases", []):
            sdk = release.get("sdk", {})
            sdk_version = sdk.get("version", "")
            if sdk_version:
                versions.append({
                    "version":    sdk_version,
                    "lts":        is_lts,
                    "prerelease": "preview" in sdk_version or "rc" in sdk_version,
                })

    return versions

# ---------------------------------------------------------------------------
# download_url — Microsoft CDN
# ---------------------------------------------------------------------------

def _dotnet_rid(ctx):
    """Map vx platform to .NET Runtime Identifier (RID)."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

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

    os = ctx["platform"]["os"]

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

def install_layout(ctx, version):
    os = ctx["platform"]["os"]
    exe = "dotnet.exe" if os == "windows" else "dotnet"
    return {
        "type":             "archive",
        "strip_prefix":     "",   # .NET SDK archives have flat layout
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "DOTNET_ROOT": install_dir,
        "PATH":        install_dir,
    }

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
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
    return "{vx_home}/store/dotnet"

def get_execute_path(ctx, version):
    os = ctx["platform"]["os"]
    if os == "windows":
        return "{install_dir}/dotnet.exe"
    else:
        return "{install_dir}/dotnet"

def post_install(ctx, version, install_dir):
    return None
