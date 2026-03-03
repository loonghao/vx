# provider.star - msbuild provider
#
# MSBuild ships with both Visual Studio / MSVC BuildTools AND the .NET SDK.
# Primary detection: MSVC BuildTools installation (most common in CI/CD).
# Fallback detection: .NET SDK installation.
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star", "runtime_def", "dep_def", "system_permissions",
     "system_install_strategies", "winget_install", "choco_install")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "msbuild"
description = "Microsoft Build Engine - ships with MSVC BuildTools and .NET SDK"
homepage    = "https://docs.microsoft.com/visualstudio/msbuild"
repository  = "https://github.com/dotnet/msbuild"
license     = "MIT"
ecosystem   = "dotnet"

# ---------------------------------------------------------------------------
# System paths — ordered by priority (newest first, 64-bit first)
# Covers: VS 2022/2019, Enterprise/Professional/Community/BuildTools,
#         Program Files and Program Files (x86), x64 and amd64 sub-dirs.
# ---------------------------------------------------------------------------

_MSBUILD_PATHS = [
    # VS 2022 — Program Files (64-bit install)
    "C:/Program Files/Microsoft Visual Studio/2022/Enterprise/MSBuild/Current/Bin/amd64/MSBuild.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/Professional/MSBuild/Current/Bin/amd64/MSBuild.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/Community/MSBuild/Current/Bin/amd64/MSBuild.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/BuildTools/MSBuild/Current/Bin/amd64/MSBuild.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/Enterprise/MSBuild/Current/Bin/MSBuild.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/Professional/MSBuild/Current/Bin/MSBuild.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/Community/MSBuild/Current/Bin/MSBuild.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/BuildTools/MSBuild/Current/Bin/MSBuild.exe",
    # VS 2022 — Program Files (x86) (BuildTools default location)
    "C:/Program Files (x86)/Microsoft Visual Studio/2022/Enterprise/MSBuild/Current/Bin/amd64/MSBuild.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2022/Professional/MSBuild/Current/Bin/amd64/MSBuild.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2022/Community/MSBuild/Current/Bin/amd64/MSBuild.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools/MSBuild/Current/Bin/amd64/MSBuild.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2022/Enterprise/MSBuild/Current/Bin/MSBuild.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2022/Professional/MSBuild/Current/Bin/MSBuild.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2022/Community/MSBuild/Current/Bin/MSBuild.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools/MSBuild/Current/Bin/MSBuild.exe",
    # VS 2019 — Program Files (x86)
    "C:/Program Files (x86)/Microsoft Visual Studio/2019/Enterprise/MSBuild/Current/Bin/amd64/MSBuild.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2019/Professional/MSBuild/Current/Bin/amd64/MSBuild.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2019/Community/MSBuild/Current/Bin/amd64/MSBuild.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2019/BuildTools/MSBuild/Current/Bin/amd64/MSBuild.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2019/Enterprise/MSBuild/Current/Bin/MSBuild.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2019/Professional/MSBuild/Current/Bin/MSBuild.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2019/Community/MSBuild/Current/Bin/MSBuild.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2019/BuildTools/MSBuild/Current/Bin/MSBuild.exe",
    # .NET SDK bundled MSBuild (fallback)
    "C:/Program Files/dotnet/sdk/*/MSBuild.dll",
]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("msbuild",
        aliases          = ["msbuild.exe", "MSBuild"],
        description      = "Microsoft Build Engine",
        auto_installable = False,
        # bundled_with msvc: MSBuild is part of MSVC BuildTools.
        # dotnet is listed as an optional dep below for .NET-only scenarios.
        bundled_with     = "msvc",
        platform_constraint = {"os": ["windows"]},
        system_paths     = _MSBUILD_PATHS,
        test_commands    = [
            {"command": "{executable} -version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(exec_cmds = ["msbuild", "dotnet", "winget", "choco"])

# ---------------------------------------------------------------------------
# fetch_versions — system detection only
# ---------------------------------------------------------------------------

def fetch_versions(_ctx):
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — not directly installable
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# system_install — install via MSVC BuildTools (preferred) or dotnet SDK
# ---------------------------------------------------------------------------

_MSBUILD_WORKLOADS = (
    "--add Microsoft.VisualStudio.Workload.VCTools " +
    "--add Microsoft.VisualStudio.Workload.ManagedDesktopBuildTools " +
    "--add Microsoft.VisualStudio.Workload.NetCoreBuildTools " +
    "--includeRecommended --quiet --norestart --wait"
)

system_install = system_install_strategies([
    winget_install(
        "Microsoft.VisualStudio.2022.BuildTools",
        priority     = 100,
        install_args = _MSBUILD_WORKLOADS,
    ),
    winget_install(
        "Microsoft.DotNet.SDK.8",
        priority     = 80,
    ),
    choco_install(
        "visualstudio2022buildtools",
        priority     = 70,
        install_args = _MSBUILD_WORKLOADS,
    ),
    choco_install(
        "dotnet-sdk",
        priority     = 60,
    ),
])

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/msbuild"

def get_execute_path(_ctx, _version):
    return None

def post_install(_ctx, _version):
    return None

def environment(_ctx, _version):
    return []

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("msvc",
                reason   = "MSBuild is bundled with MSVC BuildTools (primary source)"),
        dep_def("dotnet",
                optional = True,
                reason   = "MSBuild is also bundled with .NET SDK (fallback)"),
    ]
