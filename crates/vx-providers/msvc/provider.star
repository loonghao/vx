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

def name():
    return "msvc"

def description():
    return "Microsoft Visual C++ Build Tools"

def homepage():
    return "https://visualstudio.microsoft.com/visual-cpp-build-tools/"

def repository():
    return "https://github.com/microsoft/STL"

def license():
    return "Proprietary"

def ecosystem():
    return "system"

def platforms():
    return {"os": ["windows"]}

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
    },
    {
        "name":             "nmake",
        "executable":       "nmake",
        "description":      "Microsoft Program Maintenance Utility (bundled with MSVC)",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
    },
    {
        "name":             "link",
        "executable":       "link",
        "description":      "Microsoft Linker (bundled with MSVC)",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
    },
    {
        "name":             "ml64",
        "executable":       "ml64",
        "description":      "Microsoft MASM 64-bit Assembler (bundled with MSVC)",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
    },
    {
        "name":             "lib",
        "executable":       "lib",
        "description":      "Microsoft Library Manager (bundled with MSVC)",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
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
    "exec": ["cl", "nmake", "link", "winget", "choco"],
}

# ---------------------------------------------------------------------------
# fetch_versions — system detection only (no remote API)
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """MSVC version is tied to Visual Studio installation."""
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — not directly downloadable as portable archive
# ---------------------------------------------------------------------------

def download_url(ctx, version):
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
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    # MSVC environment is set up by vcvarsall.bat; vx does not manage it directly
    return {}

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    """MSVC recommends cmake and ninja for C++ projects."""
    return [
        {"runtime": "cmake", "version": "*", "optional": True,
         "reason": "CMake is commonly used with MSVC for C++ projects"},
        {"runtime": "ninja", "version": "*", "optional": True,
         "reason": "Ninja build system works well with MSVC"},
    ]
