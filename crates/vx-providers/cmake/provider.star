# provider.star - cmake provider
#
# CMake: Cross-platform build system generator
# Inheritance pattern: Level 2 (custom download_url for cmake's naming)
#   - fetch_versions: inherited from github.star
#   - download_url:   custom (cmake-{version}-{os}-{arch}.{ext})
#
# cmake releases: https://github.com/Kitware/CMake/releases
# Asset format: cmake-{version}-{os}-{arch}.{ext}

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "cmake"

def description():
    return "Cross-platform build system generator"

def homepage():
    return "https://cmake.org"

def repository():
    return "https://github.com/Kitware/CMake"

def license():
    return "BSD-3-Clause"

def ecosystem():
    return "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "cmake",
        "executable":  "cmake",
        "description": "CMake build system",
        "aliases":     [],
        "priority":    100,
    },
    {
        "name":         "ctest",
        "executable":   "ctest",
        "description":  "CMake test driver",
        "bundled_with": "cmake",
    },
    {
        "name":         "cpack",
        "executable":   "cpack",
        "description":  "CMake packaging tool",
        "bundled_with": "cmake",
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — inherited
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("Kitware", "CMake")

# ---------------------------------------------------------------------------
# download_url — custom
#
# cmake asset naming: cmake-{version}-{os}-{arch}.{ext}
#   cmake-3.31.5-windows-x86_64.zip
#   cmake-3.31.5-macos-universal.tar.gz
#   cmake-3.31.5-linux-x86_64.tar.gz / cmake-3.31.5-linux-aarch64.tar.gz
# ---------------------------------------------------------------------------

def _cmake_platform(ctx):
    """Map platform to cmake's naming convention."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    platform_map = {
        "windows/x64":   ("windows", "x86_64",  "zip"),
        "windows/x86":   ("windows", "i386",     "zip"),
        "macos/x64":     ("macos",   "universal", "tar.gz"),
        "macos/arm64":   ("macos",   "universal", "tar.gz"),
        "linux/x64":     ("linux",   "x86_64",   "tar.gz"),
        "linux/arm64":   ("linux",   "aarch64",  "tar.gz"),
    }
    return platform_map.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the cmake download URL.

    Args:
        ctx:     Provider context
        version: Version string, e.g. "3.31.5"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _cmake_platform(ctx)
    if not platform:
        return None

    cmake_os, cmake_arch, ext = platform
    asset = "cmake-{}-{}-{}.{}".format(version, cmake_os, cmake_arch, ext)
    tag = "v{}".format(version)
    return github_asset_url("Kitware", "CMake", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    platform = _cmake_platform(ctx)
    os = ctx["platform"]["os"]
    exe = "cmake.exe" if os == "windows" else "cmake"

    if platform:
        cmake_os, cmake_arch, _ = platform
        strip_prefix = "cmake-{}-{}-{}".format(version, cmake_os, cmake_arch)
    else:
        strip_prefix = ""

    return {
        "type":             "archive",
        "strip_prefix":     strip_prefix,
        "executable_paths": ["bin/" + exe, "bin/cmake"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir + "/bin",
    }
