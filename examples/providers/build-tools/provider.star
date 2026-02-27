# provider.star - Build Tools provider
#
# A collection of cross-platform build tools:
#   - just:  A handy command runner (Makefile alternative)
#   - cmake: Cross-platform build system generator
#   - ninja: Small build system with a focus on speed
#
# Usage:
#   vx provider add ./examples/providers/build-tools/provider.star
#   vx just --version
#   vx cmake --version
#   vx ninja --version
#
# This example demonstrates:
#   1. Multiple runtimes in one provider.star
#   2. deps() — cmake recommends ninja as its build backend
#   3. system_install fallback for all three tools
#   4. Per-tool GitHub release asset naming conventions
#   5. constraints[] — version-level dependency declarations

load("@vx//stdlib:provider.star",
     "runtime_def",
     "github_permissions",
     "multi_platform_install",
     "winget_install", "choco_install", "scoop_install",
     "brew_install", "apt_install")
load("@vx//stdlib:github.star",
     "make_fetch_versions",
     "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "build-tools"
description = "Cross-platform build tools: just, cmake, ninja"
homepage    = "https://github.com/loonghao/vx"
repository  = "https://github.com/loonghao/vx"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------
runtimes = [
    # just — command runner
    runtime_def("just",
        description   = "A handy way to save and run project-specific commands",
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "just \\d+"},
        ],
    ),
    # cmake — build system generator
    # NOTE: cmake works best with a fast build backend like ninja.
    #       The deps() function below declares this recommendation.
    runtime_def("cmake",
        description   = "Cross-platform family of tools designed to build, test and package software",
        aliases       = ["cmake3"],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "cmake version \\d+"},
        ],
    ),
    # ninja — fast build backend
    runtime_def("ninja",
        description   = "A small build system with a focus on speed",
        aliases       = ["ninja-build"],
        priority      = 100,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------
permissions = github_permissions()

# ---------------------------------------------------------------------------
# Version fetching — each tool has its own GitHub repo
# ---------------------------------------------------------------------------
def fetch_versions(ctx):
    """Fetch versions for the requested runtime from its GitHub repo."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "just"

    repos = {
        "just":        ("casey",  "just"),
        "cmake":       ("Kitware", "CMake"),
        "cmake3":      ("Kitware", "CMake"),
        "ninja":       ("ninja-build", "ninja"),
        "ninja-build": ("ninja-build", "ninja"),
    }
    entry = repos.get(runtime, repos["just"])
    owner, repo = entry[0], entry[1]

    fetcher = make_fetch_versions(owner, repo)
    return fetcher(ctx)

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------
def _ext(ctx):
    return "zip" if ctx.platform.os == "windows" else "tar.gz"

# ---------------------------------------------------------------------------
# download_url — dispatches per runtime
# ---------------------------------------------------------------------------
def download_url(ctx, version):
    """Build the download URL for the requested runtime and version."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "just"
    os      = ctx.platform.os
    arch    = ctx.platform.arch

    # ── just ────────────────────────────────────────────────────────────────
    # Asset: just-{version}-{triple}.{ext}
    # e.g.  just-1.36.0-x86_64-pc-windows-msvc.zip
    if runtime == "just":
        triples = {
            "windows/x64":  "x86_64-pc-windows-msvc",
            "macos/x64":    "x86_64-apple-darwin",
            "macos/arm64":  "aarch64-apple-darwin",
            "linux/x64":    "x86_64-unknown-linux-musl",
            "linux/arm64":  "aarch64-unknown-linux-musl",
        }
        triple = triples.get("{}/{}".format(os, arch))
        if not triple:
            return None
        ext   = _ext(ctx)
        asset = "just-{}-{}.{}".format(version, triple, ext)
        return github_asset_url("casey", "just", "v" + version, asset)

    # ── cmake ────────────────────────────────────────────────────────────────
    # Asset: cmake-{version}-{platform}.{ext}
    # e.g.  cmake-3.31.0-windows-x86_64.zip
    #       cmake-3.31.0-macos-universal.tar.gz
    #       cmake-3.31.0-linux-x86_64.tar.gz
    if runtime in ("cmake", "cmake3"):
        if os == "windows":
            platform_str = "windows-x86_64"
            ext = "zip"
        elif os == "macos":
            platform_str = "macos-universal"
            ext = "tar.gz"
        elif os == "linux":
            platform_str = "linux-x86_64" if arch == "x64" else "linux-aarch64"
            ext = "tar.gz"
        else:
            return None
        asset = "cmake-{}-{}.{}".format(version, platform_str, ext)
        return github_asset_url("Kitware", "CMake", "v" + version, asset)

    # ── ninja ────────────────────────────────────────────────────────────────
    # Asset: ninja-{platform}.zip  (single binary, always zip)
    # e.g.  ninja-win.zip / ninja-mac.zip / ninja-linux.zip
    if runtime in ("ninja", "ninja-build"):
        platform_map = {
            "windows": "win",
            "macos":   "mac",
            "linux":   "linux",
        }
        platform_str = platform_map.get(os)
        if not platform_str:
            return None
        asset = "ninja-{}.zip".format(platform_str)
        return github_asset_url("ninja-build", "ninja", "v" + version, asset)

    return None

# ---------------------------------------------------------------------------
# install_layout — dispatches per runtime
# ---------------------------------------------------------------------------
def install_layout(ctx, version):
    """Return the archive extraction descriptor for the requested runtime."""
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "just"
    os      = ctx.platform.os
    arch    = ctx.platform.arch

    # ── just: archive with no subdirectory ──────────────────────────────────
    if runtime == "just":
        exe = "just.exe" if os == "windows" else "just"
        return {
            "type":             "archive",
            "strip_prefix":     "",
            "executable_paths": [exe, "just"],
        }

    # ── cmake: archive with versioned subdirectory ───────────────────────────
    if runtime in ("cmake", "cmake3"):
        if os == "windows":
            strip = "cmake-{}-windows-x86_64".format(version)
            exe   = "bin/cmake.exe"
        elif os == "macos":
            strip = "cmake-{}-macos-universal".format(version)
            exe   = "CMake.app/Contents/bin/cmake"
        else:
            strip = "cmake-{}-linux-{}".format(version, "x86_64" if arch == "x64" else "aarch64")
            exe   = "bin/cmake"
        return {
            "type":             "archive",
            "strip_prefix":     strip,
            "executable_paths": [exe, "cmake"],
        }

    # ── ninja: single binary in zip root ────────────────────────────────────
    if runtime in ("ninja", "ninja-build"):
        exe = "ninja.exe" if os == "windows" else "ninja"
        return {
            "type":             "archive",
            "strip_prefix":     "",
            "executable_paths": [exe, "ninja"],
        }

    return {"type": "archive", "strip_prefix": "", "executable_paths": []}

# ---------------------------------------------------------------------------
# system_install — package manager fallback
# ---------------------------------------------------------------------------
system_install = multi_platform_install(
    windows_strategies = [
        winget_install("Casey.Just",          priority = 90),   # just
        choco_install("just",                 priority = 80),
        winget_install("Kitware.CMake",       priority = 90),   # cmake
        choco_install("cmake",                priority = 80),
        winget_install("Ninja-build.Ninja",   priority = 90),   # ninja
        choco_install("ninja",                priority = 80),
    ],
    macos_strategies = [
        brew_install("just",   priority = 90),
        brew_install("cmake",  priority = 90),
        brew_install("ninja",  priority = 90),
    ],
    linux_strategies = [
        brew_install("just",   priority = 70),
        apt_install("cmake",   priority = 80),
        apt_install("ninja-build", priority = 80),
    ],
)

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------
def store_root(ctx):
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "just"
    return ctx.vx_home + "/store/" + runtime

def get_execute_path(ctx, version):
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "just"
    os      = ctx.platform.os
    arch    = ctx.platform.arch

    if runtime == "just":
        exe = "just.exe" if os == "windows" else "just"
        return ctx.install_dir + "/" + exe

    if runtime in ("cmake", "cmake3"):
        if os == "windows":
            return ctx.install_dir + "/bin/cmake.exe"
        elif os == "macos":
            return ctx.install_dir + "/CMake.app/Contents/bin/cmake"
        return ctx.install_dir + "/bin/cmake"

    if runtime in ("ninja", "ninja-build"):
        exe = "ninja.exe" if os == "windows" else "ninja"
        return ctx.install_dir + "/" + exe

    return ctx.install_dir + "/" + runtime

def post_install(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Environment
# ---------------------------------------------------------------------------
def environment(ctx, _version):
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "just"
    os      = ctx.platform.os

    # cmake needs its bin/ directory on PATH
    if runtime in ("cmake", "cmake3"):
        if os == "macos":
            return [env_prepend("PATH", ctx.install_dir + "/CMake.app/Contents/bin")]
        return [env_prepend("PATH", ctx.install_dir + "/bin")]

    return [env_prepend("PATH", ctx.install_dir)]

# ---------------------------------------------------------------------------
# Dependencies
#
# This is the KEY section for the "build-tools" example.
# It demonstrates how to declare runtime dependencies between tools:
#
#   - just:  standalone, no deps
#   - cmake: RECOMMENDS ninja as its build backend (faster than make)
#   - ninja: standalone, no deps
#
# The difference between "requires" and "recommends":
#   requires   → vx will auto-install the dep before running this tool
#   recommends → vx will suggest installing the dep, but won't block
# ---------------------------------------------------------------------------
def deps(ctx, version):
    runtime = ctx.runtime_name if hasattr(ctx, "runtime_name") else "just"

    # just is a standalone command runner — no runtime dependencies
    if runtime == "just":
        return []

    # cmake works with any build backend (make, ninja, etc.)
    # We RECOMMEND ninja because it is significantly faster than make,
    # but cmake can still function without it.
    if runtime in ("cmake", "cmake3"):
        return [
            {
                "runtime": "ninja",
                "version": ">=1.10",
                "reason":  "cmake uses ninja as its default build backend for faster builds",
                "optional": True,   # recommend, not require — cmake works without ninja too
            },
        ]

    # ninja is a standalone build backend — no runtime dependencies
    if runtime in ("ninja", "ninja-build"):
        return []

    return []

# ---------------------------------------------------------------------------
# constraints — version-level dependency declarations
#
# Unlike deps() which is evaluated at runtime, constraints[] is a static
# declaration that vx reads before any installation.  Use it to express
# hard version requirements that apply regardless of the installed version.
# ---------------------------------------------------------------------------
constraints = [
    {
        # cmake >= 3.20 works best with ninja >= 1.10
        "when": ">=3.20",
        "recommends": [
            {
                "runtime": "ninja",
                "version": ">=1.10",
                "reason":  "cmake 3.20+ uses ninja file API for better IDE integration",
            },
        ],
    },
]
