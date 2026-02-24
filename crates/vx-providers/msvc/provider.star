# provider.star - MSVC Build Tools provider
#
# Microsoft Visual C++ Build Tools - Windows only
# Provides: cl (compiler), nmake, link, ml64, lib
#
# MSVC cannot be downloaded as a portable archive.
# Installation is via Visual Studio Installer (winget/choco).
# Detection searches known Visual Studio installation paths.
#
# Inheritance pattern: Level 1 (fully custom, Windows-only system tool)

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

runtimes = [
    {
        "name":             "msvc",
        "executable":       "cl",
        "description":      "Microsoft Visual C++ compiler",
        "aliases":          ["cl", "vs-build-tools", "msvc-tools"],
        "priority":         100,
        "auto_installable": True,
        "platform_constraint": {"os": ["windows"]},
        "system_paths": [
            # VS 2022
            "C:/Program Files/Microsoft Visual Studio/2022/Enterprise/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
            "C:/Program Files/Microsoft Visual Studio/2022/Professional/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
            "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
            "C:/Program Files/Microsoft Visual Studio/2022/BuildTools/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
            # VS 2019
            "C:/Program Files (x86)/Microsoft Visual Studio/2019/Enterprise/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
            "C:/Program Files (x86)/Microsoft Visual Studio/2019/Professional/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
            "C:/Program Files (x86)/Microsoft Visual Studio/2019/Community/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
            "C:/Program Files (x86)/Microsoft Visual Studio/2019/BuildTools/VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe",
        ],
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":             "nmake",
        "executable":       "nmake",
        "description":      "Microsoft Program Maintenance Utility (bundled with MSVC)",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":             "link",
        "executable":       "link",
        "description":      "Microsoft Linker (bundled with MSVC)",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":             "ml64",
        "executable":       "ml64",
        "description":      "Microsoft MASM 64-bit Assembler (bundled with MSVC)",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":             "lib",
        "executable":       "lib",
        "description":      "Microsoft Library Manager (bundled with MSVC)",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":             "dumpbin",
        "executable":       "dumpbin",
        "description":      "Microsoft COFF Binary File Dumper (bundled with MSVC)",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":             "editbin",
        "executable":       "editbin",
        "description":      "Microsoft COFF Binary File Editor (bundled with MSVC)",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":             "mt",
        "executable":       "mt",
        "description":      "Microsoft Manifest Tool (Windows SDK)",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
        "system_paths": [
            "C:/Program Files (x86)/Windows Kits/10/bin/*/x64/mt.exe",
            "C:/Program Files (x86)/Windows Kits/10/bin/*/x86/mt.exe",
            "C:/Program Files (x86)/Windows Kits/8.1/bin/x64/mt.exe",
            "C:/Program Files (x86)/Windows Kits/8.1/bin/x86/mt.exe",
        ],
        "test_commands": [
            {"command": "{executable} /?", "name": "help_check"},
        ],
    },
    {
        "name":             "rc",
        "executable":       "rc",
        "description":      "Microsoft Resource Compiler (Windows SDK)",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
        "system_paths": [
            "C:/Program Files (x86)/Windows Kits/10/bin/*/x64/rc.exe",
            "C:/Program Files (x86)/Windows Kits/10/bin/*/x86/rc.exe",
            "C:/Program Files (x86)/Windows Kits/8.1/bin/x64/rc.exe",
            "C:/Program Files (x86)/Windows Kits/8.1/bin/x86/rc.exe",
        ],
        "test_commands": [
            {"command": "{executable} /?", "name": "help_check"},
        ],
    },
    {
        "name":             "signtool",
        "executable":       "signtool",
        "description":      "Microsoft Authenticode signing tool (Windows SDK)",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
        "system_paths": [
            # Windows SDK 10 (x64)
            "C:/Program Files (x86)/Windows Kits/10/bin/*/x64/signtool.exe",
            # Windows SDK 10 (x86)
            "C:/Program Files (x86)/Windows Kits/10/bin/*/x86/signtool.exe",
            # Windows SDK 8.1
            "C:/Program Files (x86)/Windows Kits/8.1/bin/x64/signtool.exe",
            "C:/Program Files (x86)/Windows Kits/8.1/bin/x86/signtool.exe",
            # Bundled with Visual Studio
            "C:/Program Files/Microsoft Visual Studio/2022/Enterprise/SDK/ScopedNetFxSDK/*/bin/signtool.exe",
            "C:/Program Files/Microsoft Visual Studio/2022/Professional/SDK/ScopedNetFxSDK/*/bin/signtool.exe",
            "C:/Program Files/Microsoft Visual Studio/2022/Community/SDK/ScopedNetFxSDK/*/bin/signtool.exe",
            "C:/Program Files/Microsoft Visual Studio/2022/BuildTools/SDK/ScopedNetFxSDK/*/bin/signtool.exe",
        ],
        "test_commands": [
            {"command": "{executable} /?", "name": "help_check"},
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
    "exec": ["cl", "nmake", "link", "dumpbin", "editbin", "mt", "rc", "signtool", "winget", "choco"],
}

# ---------------------------------------------------------------------------
# fetch_versions — system detection only (no remote API)
# ---------------------------------------------------------------------------

def fetch_versions(_ctx):
    """MSVC version is tied to Visual Studio installation."""
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — not directly downloadable as portable archive
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    """MSVC is installed via Visual Studio Installer, not a portable archive."""
    return None

# ---------------------------------------------------------------------------
# system_install — winget/choco strategies
# ---------------------------------------------------------------------------

system_install = {
    "strategies": [
        {
            "type":         "package_manager",
            "manager":      "winget",
            "package":      "Microsoft.VisualStudio.2022.BuildTools",
            "install_args": "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive --wait",
            "platforms":    ["windows"],
            "priority":     100,
        },
        {
            "type":         "package_manager",
            "manager":      "choco",
            "package":      "visualstudio2022buildtools",
            "params":       "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive --wait",
            "platforms":    ["windows"],
            "priority":     80,
        },
    ],
}

# ---------------------------------------------------------------------------
# store_root — not managed by vx (installed via Visual Studio Installer)
# ---------------------------------------------------------------------------

def store_root(_ctx, _version):
    """MSVC is installed via Visual Studio Installer — no vx store root."""
    return None

# ---------------------------------------------------------------------------
# get_execute_path — resolve cl.exe via system_paths
# ---------------------------------------------------------------------------

def get_execute_path(_ctx, _version, install_dir):
    """MSVC is located via system_paths; no vx-managed install_dir."""
    return None

# ---------------------------------------------------------------------------
# post_install — nothing to do
# ---------------------------------------------------------------------------

def post_install(_ctx, _version):
    """No post-install steps required for MSVC."""
    return []

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(_ctx, _version):
    # MSVC environment is set up by vcvarsall.bat; vx does not manage it directly
    return []

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, version):
    """MSVC recommends cmake and ninja for C++ projects."""
    return [
        {"runtime": "cmake", "version": "*", "optional": True,
         "reason": "CMake is commonly used with MSVC for C++ projects"},
        {"runtime": "ninja", "version": "*", "optional": True,
         "reason": "Ninja build system works well with MSVC"},
    ]
