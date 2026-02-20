# provider.star - msbuild provider
#
# Microsoft Build Engine - bundled with .NET SDK
# Inheritance pattern: Level 1 (fully custom, bundled with dotnet, not directly installable)
#
# MSBuild is bundled with .NET SDK (cross-platform) and Visual Studio (Windows-only).
# It cannot be installed independently - vx delegates to `dotnet msbuild`.

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "msbuild"

def description():
    return "Microsoft Build Engine - bundled with .NET SDK"

def homepage():
    return "https://docs.microsoft.com/visualstudio/msbuild"

def repository():
    return "https://github.com/dotnet/msbuild"

def license():
    return "MIT"

def ecosystem():
    return "dotnet"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":             "msbuild",
        "executable":       "msbuild",
        "description":      "Build .NET, C++, and other projects",
        "aliases":          ["msbuild.exe"],
        "priority":         100,
        "auto_installable": False,
        "bundled_with":     "dotnet",
        "system_paths":     [
            # Visual Studio 2022
            "C:/Program Files/Microsoft Visual Studio/2022/Enterprise/MSBuild/Current/Bin/MSBuild.exe",
            "C:/Program Files/Microsoft Visual Studio/2022/Professional/MSBuild/Current/Bin/MSBuild.exe",
            "C:/Program Files/Microsoft Visual Studio/2022/Community/MSBuild/Current/Bin/MSBuild.exe",
            "C:/Program Files/Microsoft Visual Studio/2022/BuildTools/MSBuild/Current/Bin/MSBuild.exe",
            # Visual Studio 2019
            "C:/Program Files (x86)/Microsoft Visual Studio/2019/Enterprise/MSBuild/Current/Bin/MSBuild.exe",
            "C:/Program Files (x86)/Microsoft Visual Studio/2019/Professional/MSBuild/Current/Bin/MSBuild.exe",
            "C:/Program Files (x86)/Microsoft Visual Studio/2019/Community/MSBuild/Current/Bin/MSBuild.exe",
            "C:/Program Files (x86)/Microsoft Visual Studio/2019/BuildTools/MSBuild/Current/Bin/MSBuild.exe",
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": [],
    "fs":   [
        "C:/Program Files/Microsoft Visual Studio",
        "C:/Program Files (x86)/Microsoft Visual Studio",
    ],
    "exec": ["dotnet", "msbuild"],
}

# ---------------------------------------------------------------------------
# fetch_versions — system detection only
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """MSBuild version is tied to .NET SDK / Visual Studio."""
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — not directly installable
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """MSBuild is bundled with .NET SDK — install dotnet instead."""
    return None

# ---------------------------------------------------------------------------
# deps — requires dotnet
# ---------------------------------------------------------------------------

def deps(ctx, version):
    """MSBuild is bundled with .NET SDK."""
    return [{"runtime": "dotnet", "version": "*"}]

# ---------------------------------------------------------------------------
# store_root — not managed by vx (bundled with dotnet / Visual Studio)
# ---------------------------------------------------------------------------

def store_root(ctx, version):
    """MSBuild is bundled with .NET SDK or Visual Studio — no vx store root."""
    return None

# ---------------------------------------------------------------------------
# get_execute_path — resolve MSBuild executable
# ---------------------------------------------------------------------------

def get_execute_path(ctx, version, install_dir):
    """Return the path to the MSBuild executable.

    MSBuild is not installed by vx directly; it is located via system_paths
    or delegated to `dotnet msbuild`.
    """
    return None

# ---------------------------------------------------------------------------
# post_install — nothing to do
# ---------------------------------------------------------------------------

def post_install(ctx, version, install_dir):
    """No post-install steps required for MSBuild."""
    return []

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {}
