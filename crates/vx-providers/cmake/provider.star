# provider.star - CMake provider
#
# CMake - Cross-platform build system generator
# Unix platforms can use upstream archives.
# Windows prefers system package managers.
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "github_permissions",
     "system_install_strategies", "winget_install", "choco_install",
     "brew_install", "apt_install")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "cmake"
description = "CMake - Cross-platform build system generator"
homepage    = "https://cmake.org"
repository  = "https://github.com/Kitware/CMake"
license     = "BSD-3-Clause"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("cmake",
        system_paths = [
            "C:/Program Files/CMake/bin/cmake.exe",
            "C:/Program Files (x86)/CMake/bin/cmake.exe",
            "/usr/local/bin/cmake",
            "/usr/bin/cmake",
            "/opt/homebrew/bin/cmake",
        ],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "cmake version"},
        ],
    ),
    bundled_runtime_def("ctest", "cmake",
        description = "CMake test driver",
    ),
    bundled_runtime_def("cpack", "cmake",
        description = "CMake packaging tool",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(exec_cmds = ["winget", "choco", "brew", "apt"])

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("Kitware", "CMake")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_CMAKE_PLATFORMS = {
    "windows/x64":   ("windows", "x86_64", "zip"),
    "windows/arm64": ("windows", "arm64",  "zip"),
    "macos/x64":     ("Darwin",  "x86_64", "tar.gz"),
    "macos/arm64":   ("Darwin",  "arm64",  "tar.gz"),
    "linux/x64":     ("linux",   "x86_64", "tar.gz"),
    "linux/arm64":   ("linux",   "aarch64", "tar.gz"),
}


def _cmake_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _CMAKE_PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    if ctx.platform.os == "windows":
        return None
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
# system_install
# ---------------------------------------------------------------------------

system_install = system_install_strategies([
    winget_install("Kitware.CMake", priority = 90),
    choco_install("cmake", priority = 80),
    brew_install("cmake", priority = 70),
    apt_install("cmake", priority = 70),
])

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/cmake"


def get_execute_path(ctx, _version):
    exe = "cmake.exe" if ctx.platform.os == "windows" else "cmake"
    return ctx.install_dir + "/bin/" + exe


def environment(ctx, _version):
    return [{"op": "prepend", "key": "PATH", "value": ctx.install_dir + "/bin"}]


def post_install(_ctx, _version):
    return None


def deps(_ctx, _version):
    return []
