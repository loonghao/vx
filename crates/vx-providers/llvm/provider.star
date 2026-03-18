# provider.star - LLVM/Clang toolchain provider
#
# LLVM provides: clang, clang-cl, clang-format, clang-tidy, lld, lld-link,
#                llvm-ar, llvm-nm, llvm-objdump, llvm-ranlib, llvm-strip
#
# Downloads from GitHub releases: llvm/llvm-project

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "dep_def",
     "github_permissions", "fetch_versions_with_tag_prefix",
     "system_install_strategies", "winget_install", "brew_install")
load("@vx//stdlib:env.star", "env_prepend", "env_set")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "llvm"
description = "LLVM/Clang compiler infrastructure and toolchain"
homepage    = "https://llvm.org"
repository  = "https://github.com/llvm/llvm-project"
license     = "Apache-2.0 WITH LLVM-exception"
ecosystem   = "cpp"
aliases     = ["clang"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

_LLVM_WIN_PATHS = [
    "C:/Program Files/LLVM/bin/clang.exe",
    "C:/Program Files (x86)/LLVM/bin/clang.exe",
]

_CLANG_CL_WIN_PATHS = [
    "C:/Program Files/LLVM/bin/clang-cl.exe",
    "C:/Program Files (x86)/LLVM/bin/clang-cl.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/Enterprise/VC/Tools/Llvm/x64/bin/clang-cl.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/Professional/VC/Tools/Llvm/x64/bin/clang-cl.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/Llvm/x64/bin/clang-cl.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/BuildTools/VC/Tools/Llvm/x64/bin/clang-cl.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2022/Enterprise/VC/Tools/Llvm/x64/bin/clang-cl.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2022/Professional/VC/Tools/Llvm/x64/bin/clang-cl.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2022/Community/VC/Tools/Llvm/x64/bin/clang-cl.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools/VC/Tools/Llvm/x64/bin/clang-cl.exe",
]

runtimes = [
    runtime_def("llvm",
        executable   = "clang",
        aliases      = ["clang", "llvm-toolchain"],
        system_paths = _LLVM_WIN_PATHS,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "clang version \\d+"},
        ],
    ),
    bundled_runtime_def("clang++", bundled_with = "llvm"),
    bundled_runtime_def("clang-format", bundled_with = "llvm",
        description = "LLVM code formatter"),
    bundled_runtime_def("clang-tidy", bundled_with = "llvm",
        description = "LLVM static analyzer and linter"),
    runtime_def("clang-cl",
        bundled_with  = "llvm",
        description   = "MSVC-compatible Clang frontend (Windows)",
        platform_constraint = {"os": ["windows"]},
        system_paths  = _CLANG_CL_WIN_PATHS,
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "clang version \\d+"},
        ],
    ),
    bundled_runtime_def("lld", bundled_with = "llvm",
        description = "LLVM linker"),
    bundled_runtime_def("lld-link", bundled_with = "llvm",
        description = "LLVM linker (MSVC-compatible interface)"),
    bundled_runtime_def("llvm-ar", bundled_with = "llvm",
        description = "LLVM archiver"),
    bundled_runtime_def("llvm-nm", bundled_with = "llvm",
        description = "LLVM symbol lister"),
    bundled_runtime_def("llvm-objdump", bundled_with = "llvm",
        description = "LLVM object file dumper"),
    bundled_runtime_def("llvm-ranlib", bundled_with = "llvm",
        description = "LLVM archive index generator"),
    bundled_runtime_def("llvm-strip", bundled_with = "llvm",
        description = "LLVM symbol stripper"),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["github.com"])

# ---------------------------------------------------------------------------
# fetch_versions — tags: llvmorg-{version}
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix(
    "llvm", "llvm-project", tag_prefix = "llvmorg-"
)

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

# Old format (LLVM < 20): clang+llvm-{version}-{triple}.tar.xz
_OLD_PLATFORMS = {
    "windows/x64":   ("x86_64-pc-windows-msvc", "tar.xz"),
    "macos/x64":     ("x86_64-apple-darwin", "tar.xz"),
    "macos/arm64":   ("arm64-apple-macos11", "tar.xz"),
    "linux/x64":     ("x86_64-linux-gnu-ubuntu-22.04", "tar.xz"),
    "linux/arm64":   ("aarch64-linux-gnu", "tar.xz"),
}

# New format (LLVM >= 20): LLVM-{version}-{OS}-{Arch}.tar.xz
# Note: Windows still uses the old clang+llvm format even in >= 20
_NEW_PLATFORMS = {
    "macos/arm64":   ("macOS", "ARM64"),
    "linux/x64":     ("Linux", "X64"),
    "linux/arm64":   ("Linux", "ARM64"),
}

def _major_version(version):
    """Extract major version number from a version string like '22.1.1'."""
    parts = version.split(".")
    if len(parts) > 0:
        return int(parts[0])
    return 0

def _uses_new_format(version):
    """LLVM >= 20 uses new asset naming for Linux/macOS."""
    major = _major_version(version)
    return major >= 20

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    tag = "llvmorg-{}".format(version)

    if _uses_new_format(version):
        # Windows still uses old format even in >= 20
        if ctx.platform.os == "windows":
            old = _OLD_PLATFORMS.get(key)
            if not old:
                return None
            triple, ext = old
            asset = "clang+llvm-{}-{}.{}".format(version, triple, ext)
        else:
            new = _NEW_PLATFORMS.get(key)
            if not new:
                return None
            os_name, arch_name = new
            asset = "LLVM-{}-{}-{}.tar.xz".format(version, os_name, arch_name)
    else:
        old = _OLD_PLATFORMS.get(key)
        if not old:
            return None
        triple, ext = old
        asset = "clang+llvm-{}-{}.{}".format(version, triple, ext)

    return "https://github.com/llvm/llvm-project/releases/download/{}/{}".format(tag, asset)

# ---------------------------------------------------------------------------
# install_layout — archive with bin/ subdirectory
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)

    if _uses_new_format(version) and ctx.platform.os != "windows":
        new = _NEW_PLATFORMS.get(key)
        if not new:
            return None
        os_name, arch_name = new
        strip_prefix = "LLVM-{}-{}-{}".format(version, os_name, arch_name)
    else:
        old = _OLD_PLATFORMS.get(key)
        if not old:
            return None
        triple, _ext = old
        strip_prefix = "clang+llvm-{}-{}".format(version, triple)

    if ctx.platform.os == "windows":
        exe_paths = [
            "bin/clang.exe", "bin/clang++.exe", "bin/clang-cl.exe",
            "bin/clang-format.exe", "bin/clang-tidy.exe",
            "bin/lld.exe", "bin/lld-link.exe",
            "bin/llvm-ar.exe", "bin/llvm-nm.exe", "bin/llvm-objdump.exe",
            "bin/llvm-ranlib.exe", "bin/llvm-strip.exe",
        ]
    else:
        exe_paths = [
            "bin/clang", "bin/clang++", "bin/clang-cl",
            "bin/clang-format", "bin/clang-tidy",
            "bin/lld", "bin/lld-link",
            "bin/llvm-ar", "bin/llvm-nm", "bin/llvm-objdump",
            "bin/llvm-ranlib", "bin/llvm-strip",
        ]
    return {
        "type":             "archive",
        "strip_prefix":     strip_prefix,
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# system_install — fallback via package managers
# ---------------------------------------------------------------------------

system_install = system_install_strategies([
    winget_install("LLVM.LLVM", priority = 90),
    brew_install("llvm", priority = 85),
])

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/llvm"


def get_execute_path(ctx, _version):
    exe = "clang.exe" if ctx.platform.os == "windows" else "clang"
    return ctx.install_dir + "/bin/" + exe


def post_install(_ctx, _version):
    return None


def environment(ctx, _version):
    return [
        env_prepend("PATH", ctx.install_dir + "/bin"),
        env_set("LLVM_HOME", ctx.install_dir),
    ]

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("cmake", optional = True,
                reason = "CMake is commonly used to build projects with Clang"),
        dep_def("ninja", optional = True,
                reason = "Ninja provides faster builds with Clang"),
    ]
