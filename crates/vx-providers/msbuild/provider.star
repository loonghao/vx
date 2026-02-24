# provider.star - msbuild provider
#
# MSBuild is bundled with .NET SDK — not directly installable.
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star", "runtime_def", "dep_def", "system_permissions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "msbuild"
description = "Microsoft Build Engine - bundled with .NET SDK"
homepage    = "https://docs.microsoft.com/visualstudio/msbuild"
repository  = "https://github.com/dotnet/msbuild"
license     = "MIT"
ecosystem   = "dotnet"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("msbuild",
        aliases         = ["msbuild.exe"],
        auto_installable = False,
        bundled_with    = "dotnet",
        system_paths    = [
            "C:/Program Files/Microsoft Visual Studio/2022/Enterprise/MSBuild/Current/Bin/MSBuild.exe",
            "C:/Program Files/Microsoft Visual Studio/2022/Professional/MSBuild/Current/Bin/MSBuild.exe",
            "C:/Program Files/Microsoft Visual Studio/2022/Community/MSBuild/Current/Bin/MSBuild.exe",
            "C:/Program Files/Microsoft Visual Studio/2022/BuildTools/MSBuild/Current/Bin/MSBuild.exe",
            "C:/Program Files (x86)/Microsoft Visual Studio/2019/Enterprise/MSBuild/Current/Bin/MSBuild.exe",
            "C:/Program Files (x86)/Microsoft Visual Studio/2019/Professional/MSBuild/Current/Bin/MSBuild.exe",
            "C:/Program Files (x86)/Microsoft Visual Studio/2019/Community/MSBuild/Current/Bin/MSBuild.exe",
            "C:/Program Files (x86)/Microsoft Visual Studio/2019/BuildTools/MSBuild/Current/Bin/MSBuild.exe",
        ],
        test_commands = [
            {"command": "{executable} -version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(exec_cmds = ["dotnet", "msbuild"])

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
    return [dep_def("dotnet", reason = "MSBuild is bundled with .NET SDK")]
