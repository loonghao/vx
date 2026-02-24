# provider.star - MSVC Build Tools provider
#
# Windows-only. Provides: cl, nmake, link, ml64, lib, dumpbin, editbin, mt, rc, signtool
# Not directly downloadable — installed via Visual Studio Installer.
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "dep_def",
     "system_permissions",
     "system_install_strategies", "winget_install", "choco_install")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "msvc"
description = "Microsoft Visual C++ Build Tools"
homepage    = "https://visualstudio.microsoft.com/visual-cpp-build-tools/"
repository  = "https://github.com/microsoft/STL"
license     = "Proprietary"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

_MSVC_PATHS = [
    "C:/Program Files/Microsoft Visual Studio/2022/Enterprise/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/Professional/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/BuildTools/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2019/Enterprise/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2019/Professional/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2019/Community/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2019/BuildTools/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
]

_WIN_SDK_MT = [
    "C:/Program Files (x86)/Windows Kits/10/bin/*/x64/mt.exe",
    "C:/Program Files (x86)/Windows Kits/10/bin/*/x86/mt.exe",
    "C:/Program Files (x86)/Windows Kits/8.1/bin/x64/mt.exe",
]

_WIN_SDK_RC = [
    "C:/Program Files (x86)/Windows Kits/10/bin/*/x64/rc.exe",
    "C:/Program Files (x86)/Windows Kits/10/bin/*/x86/rc.exe",
]

_WIN_SDK_SIGNTOOL = [
    "C:/Program Files (x86)/Windows Kits/10/bin/*/x64/signtool.exe",
    "C:/Program Files (x86)/Windows Kits/10/bin/*/x86/signtool.exe",
    "C:/Program Files (x86)/Windows Kits/8.1/bin/x64/signtool.exe",
]

runtimes = [
    runtime_def("msvc",
        aliases             = ["cl", "vs-build-tools", "msvc-tools"],
        auto_installable    = True,
        platform_constraint = {"os": ["windows"]},
        system_paths        = _MSVC_PATHS,
        test_commands       = [{"command": "{executable} --version", "name": "version_check"}],
    ),
    bundled_runtime_def("nmake",    bundled_with = "msvc",
        auto_installable = False, platform_constraint = {"os": ["windows"]}),
    bundled_runtime_def("link",     bundled_with = "msvc",
        auto_installable = False, platform_constraint = {"os": ["windows"]}),
    bundled_runtime_def("ml64",     bundled_with = "msvc",
        auto_installable = False, platform_constraint = {"os": ["windows"]}),
    bundled_runtime_def("lib",      bundled_with = "msvc",
        auto_installable = False, platform_constraint = {"os": ["windows"]}),
    bundled_runtime_def("dumpbin",  bundled_with = "msvc",
        auto_installable = False, platform_constraint = {"os": ["windows"]}),
    bundled_runtime_def("editbin",  bundled_with = "msvc",
        auto_installable = False, platform_constraint = {"os": ["windows"]}),
    runtime_def("mt",
        bundled_with        = "msvc",
        auto_installable    = False,
        platform_constraint = {"os": ["windows"]},
        system_paths        = _WIN_SDK_MT,
        test_commands       = [{"command": "{executable} /?", "name": "help_check"}],
    ),
    runtime_def("rc",
        bundled_with        = "msvc",
        auto_installable    = False,
        platform_constraint = {"os": ["windows"]},
        system_paths        = _WIN_SDK_RC,
        test_commands       = [{"command": "{executable} /?", "name": "help_check"}],
    ),
    runtime_def("signtool",
        bundled_with        = "msvc",
        auto_installable    = False,
        platform_constraint = {"os": ["windows"]},
        system_paths        = _WIN_SDK_SIGNTOOL,
        test_commands       = [{"command": "{executable} /?", "name": "help_check"}],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(
    exec_cmds = ["cl", "nmake", "link", "dumpbin", "editbin", "mt", "rc", "signtool", "winget", "choco"],
)

# ---------------------------------------------------------------------------
# fetch_versions — system detection only
# ---------------------------------------------------------------------------

def fetch_versions(_ctx):
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — not directly downloadable
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# system_install
# ---------------------------------------------------------------------------

system_install = system_install_strategies([
    winget_install(
        "Microsoft.VisualStudio.2022.BuildTools",
        priority     = 100,
        install_args = "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive --wait",
    ),
    choco_install(
        "visualstudio2022buildtools",
        priority     = 80,
        install_args = "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive --wait",
    ),
])

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/msvc"

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
        dep_def("cmake", optional = True, reason = "CMake is commonly used with MSVC"),
        dep_def("ninja", optional = True, reason = "Ninja build system works well with MSVC"),
    ]
