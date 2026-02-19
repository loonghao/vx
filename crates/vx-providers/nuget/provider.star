# provider.star - nuget provider
#
# NuGet: The package manager for .NET
# Inheritance pattern: Level 3 (custom fetch + download, Windows-only binary)
#   - fetch_versions: custom (GitHub releases)
#   - download_url:   custom (direct binary from nuget.org, Windows-only)
#
# nuget.exe is Windows-only; on macOS/Linux use `dotnet nuget`
# Download: https://dist.nuget.org/win-x86-commandline/v{version}/nuget.exe

load("@vx//stdlib:github.star", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "nuget"

def description():
    return "NuGet - The package manager for .NET"

def homepage():
    return "https://www.nuget.org/"

def repository():
    return "https://github.com/NuGet/NuGet.Client"

def license():
    return "Apache-2.0"

def ecosystem():
    return "dotnet"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "nuget",
        "executable":  "nuget",
        "description": "NuGet command-line tool",
        "aliases":     ["nuget-cli"],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com", "dist.nuget.org"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — inherited from GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("NuGet", "NuGet.Client")

# ---------------------------------------------------------------------------
# download_url — custom
#
# nuget.exe is Windows-only, downloaded from nuget.org CDN:
#   https://dist.nuget.org/win-x86-commandline/v{version}/nuget.exe
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build the nuget download URL.

    nuget.exe is Windows-only. On macOS/Linux, use `dotnet nuget` instead.

    Args:
        ctx:     Provider context
        version: Version string, e.g. "6.12.1"

    Returns:
        Download URL string, or None if not Windows
    """
    os = ctx["platform"]["os"]
    if os != "windows":
        return None

    return "https://dist.nuget.org/win-x86-commandline/v{}/nuget.exe".format(version)

# ---------------------------------------------------------------------------
# install_layout — binary (single file)
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    return {
        "type":       "binary",
        "target_name": "nuget.exe",
        "target_dir":  "bin",
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir + "/bin",
    }
