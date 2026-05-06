# provider.star - WiX Toolset provider
#
# WiX Toolset - Windows Installer XML toolset for building MSI packages.
# Windows-only. WiX v4+ is distributed as a .NET tool via NuGet/dotnet.
# WiX v3 provides standalone binaries on GitHub.
#
# WiX v4: installed via `dotnet tool install --global wix`
# WiX v3: https://github.com/wixtoolset/wix3/releases
#
# This provider covers both:
#   - wix    (v4 CLI, via dotnet tool)
#   - candle (v3 compiler)
#   - light  (v3 linker)
#   - heat   (v3 harvester)
#   - torch  (v3 transform builder)
#   - smoke  (v3 validator)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "dep_def",
     "github_permissions",
     "system_install_strategies", "winget_install", "choco_install")
load("@vx//stdlib:github.star", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "wix"
description = "WiX Toolset - Build Windows installation packages"
homepage    = "https://wixtoolset.org/"
repository  = "https://github.com/wixtoolset/wix"
license     = "MS-RL"
ecosystem   = "dotnet"

# ---------------------------------------------------------------------------
# System paths for WiX v4 (dotnet tool or standalone install)
# ---------------------------------------------------------------------------

_WIX4_PATHS = [
    "C:/Program Files/WiX Toolset v4*/bin/wix.exe",
    "C:/Program Files (x86)/WiX Toolset v4*/bin/wix.exe",
    "C:/Program Files/WiX Toolset v5*/bin/wix.exe",
    "C:/Program Files (x86)/WiX Toolset v5*/bin/wix.exe",
]

# ---------------------------------------------------------------------------
# System paths for WiX v3 (standalone install)
# ---------------------------------------------------------------------------

_WIX3_PATHS = [
    "C:/Program Files (x86)/WiX Toolset v3.*/bin/candle.exe",
    "C:/Program Files/WiX Toolset v3.*/bin/candle.exe",
]

_WIX3_LIGHT_PATHS = [
    "C:/Program Files (x86)/WiX Toolset v3.*/bin/light.exe",
    "C:/Program Files/WiX Toolset v3.*/bin/light.exe",
]

_WIX3_HEAT_PATHS = [
    "C:/Program Files (x86)/WiX Toolset v3.*/bin/heat.exe",
    "C:/Program Files/WiX Toolset v3.*/bin/heat.exe",
]

_WIX3_TORCH_PATHS = [
    "C:/Program Files (x86)/WiX Toolset v3.*/bin/torch.exe",
    "C:/Program Files/WiX Toolset v3.*/bin/torch.exe",
]

_WIX3_SMOKE_PATHS = [
    "C:/Program Files (x86)/WiX Toolset v3.*/bin/smoke.exe",
    "C:/Program Files/WiX Toolset v3.*/bin/smoke.exe",
]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    # WiX v4 CLI (primary, installed via dotnet tool or winget)
    runtime_def("wix",
        aliases             = ["wix4", "wix-toolset"],
        auto_installable    = True,
        platform_constraint = {"os": ["windows"]},
        system_paths        = _WIX4_PATHS,
        test_commands       = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+"},
        ],
    ),
    # WiX v3 tools (legacy, system-installed)
    runtime_def("candle",
        description         = "WiX v3 compiler - compiles .wxs source files to .wixobj",
        bundled_with        = "wix",
        auto_installable    = False,
        platform_constraint = {"os": ["windows"]},
        system_paths        = _WIX3_PATHS,
        test_commands       = [{"command": "{executable} /?", "name": "help_check"}],
    ),
    runtime_def("light",
        description         = "WiX v3 linker - links .wixobj files into MSI/MSM packages",
        bundled_with        = "wix",
        auto_installable    = False,
        platform_constraint = {"os": ["windows"]},
        system_paths        = _WIX3_LIGHT_PATHS,
        test_commands       = [{"command": "{executable} /?", "name": "help_check"}],
    ),
    runtime_def("heat",
        description         = "WiX v3 harvester - generates WiX authoring from various inputs",
        bundled_with        = "wix",
        auto_installable    = False,
        platform_constraint = {"os": ["windows"]},
        system_paths        = _WIX3_HEAT_PATHS,
        test_commands       = [{"command": "{executable} /?", "name": "help_check"}],
    ),
    runtime_def("torch",
        description         = "WiX v3 transform builder - creates MST transform files",
        bundled_with        = "wix",
        auto_installable    = False,
        platform_constraint = {"os": ["windows"]},
        system_paths        = _WIX3_TORCH_PATHS,
        test_commands       = [{"command": "{executable} /?", "name": "help_check"}],
    ),
    runtime_def("smoke",
        description         = "WiX v3 validator - validates MSI/MSM packages",
        bundled_with        = "wix",
        auto_installable    = False,
        platform_constraint = {"os": ["windows"]},
        system_paths        = _WIX3_SMOKE_PATHS,
        test_commands       = [{"command": "{executable} /?", "name": "help_check"}],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(
    extra_hosts = ["www.nuget.org", "api.nuget.org"],
)

# ---------------------------------------------------------------------------
# fetch_versions — from GitHub releases (wix v4+)
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("vx-org", "mirrors", tag_prefix = "wix-")

# ---------------------------------------------------------------------------
# download_url
#
# WiX v4 is distributed as a .NET global tool via NuGet.
# There is no standalone binary download — it must be installed via:
#   dotnet tool install --global wix --version <version>
#
# WiX v3 has standalone binaries but we handle those via system_paths above.
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    # WiX v4 is installed via dotnet tool, not direct download
    return None

# ---------------------------------------------------------------------------
# system_install — WiX v4 via dotnet tool, WiX v3 via winget/choco
# ---------------------------------------------------------------------------

system_install = system_install_strategies([
    winget_install(
        "WiXToolset.WiX",
        priority = 90,
    ),
    choco_install(
        "wixtoolset",
        priority = 80,
    ),
])

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/wix"

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
        dep_def("dotnet",
                reason   = "WiX v4 is installed as a .NET global tool"),
        dep_def("msvc",
                optional = True,
                reason   = "MSVC is recommended for building native bootstrapper applications"),
    ]
