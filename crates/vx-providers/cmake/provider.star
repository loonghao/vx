# provider.star - cmake provider
#
# CMake: Cross-platform build system generator
# Asset: cmake-{version}-{os}-{arch}.{ext}
# Bundled runtimes: ctest, cpack
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "cmake"
description = "Cross-platform build system generator"
homepage    = "https://cmake.org"
repository  = "https://github.com/Kitware/CMake"
license     = "BSD-3-Clause"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("cmake",
        version_pattern = "cmake version",
    ),
    bundled_runtime_def("ctest", bundled_with = "cmake"),
    bundled_runtime_def("cpack", bundled_with = "cmake"),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("Kitware", "CMake")

# ---------------------------------------------------------------------------
# Platform helpers
# cmake asset: cmake-{version}-{os}-{arch}.{ext}
# macOS uses "universal" for both x64 and arm64
# ---------------------------------------------------------------------------

_CMAKE_PLATFORMS = {
    "windows/x64":  ("windows", "x86_64",   "zip"),
    "windows/x86":  ("windows", "i386",     "zip"),
    "macos/x64":    ("macos",   "universal", "tar.gz"),
    "macos/arm64":  ("macos",   "universal", "tar.gz"),
    "linux/x64":    ("linux",   "x86_64",   "tar.gz"),
    "linux/arm64":  ("linux",   "aarch64",  "tar.gz"),
}

def _cmake_platform(ctx):
    return _CMAKE_PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _cmake_platform(ctx)
    if not platform:
        return None
    cmake_os, cmake_arch, ext = platform[0], platform[1], platform[2]
    asset = "cmake-{}-{}-{}.{}".format(version, cmake_os, cmake_arch, ext)
    return github_asset_url("Kitware", "CMake", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — strip top-level "cmake-{version}-{os}-{arch}/" dir
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    platform = _cmake_platform(ctx)
    exe      = "cmake.exe" if ctx.platform.os == "windows" else "cmake"
    strip    = ""
    if platform:
        cmake_os, cmake_arch, _ = platform[0], platform[1], platform[2]
        strip = "cmake-{}-{}-{}".format(version, cmake_os, cmake_arch)
    return {
        "type":             "archive",
        "strip_prefix":     strip,
        "executable_paths": ["bin/" + exe, "bin/cmake"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/cmake"

def get_execute_path(ctx, _version):
    exe = "cmake.exe" if ctx.platform.os == "windows" else "cmake"
    return ctx.install_dir + "/bin/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def deps(_ctx, _version):
    return []
