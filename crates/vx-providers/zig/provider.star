# provider.star - Zig programming language provider
#
# Downloads from ziglang.org (not GitHub releases).
# Asset naming: zig-{arch}-{os}-{version}.{ext}  (arch BEFORE os, unusual)
# Windows: .zip, others: .tar.xz
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star", "runtime_def", "system_permissions")
load("@vx//stdlib:github.star",   "make_fetch_versions")
load("@vx//stdlib:env.star",      "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "zig"
description = "Zig - A general-purpose programming language and toolchain"
homepage    = "https://ziglang.org"
repository  = "https://github.com/ziglang/zig"
license     = "MIT"
ecosystem   = "zig"

# Supported package prefixes for ecosystem:package syntax (RFC 0027)
# Enables `vx zig:<package>` for Zig package management via `zig build`
package_prefixes = ["zig"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("zig",
        test_commands = [
            {"command": "{executable} version", "name": "version_check",
             "expected_output": "^\\d+\\.\\d+"},
            {"command": "{executable} zen", "name": "zen_check",
             "expected_output": "Communicate intent"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(extra_hosts = ["ziglang.org"])

# ---------------------------------------------------------------------------
# fetch_versions — ziglang/zig GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("ziglang", "zig")

# ---------------------------------------------------------------------------
# Platform helpers
# Zig asset: zig-{arch}-{os}-{version}.{ext}  (arch BEFORE os)
# ---------------------------------------------------------------------------

_ZIG_ARCH = {"x64": "x86_64", "arm64": "aarch64", "x86": "x86", "arm": "armv7a"}
_ZIG_OS   = {"windows": "windows", "macos": "macos", "linux": "linux"}

def _zig_arch(ctx):
    return _ZIG_ARCH.get(ctx.platform.arch, "x86_64")

def _zig_os(ctx):
    return _ZIG_OS.get(ctx.platform.os, "linux")

# ---------------------------------------------------------------------------
# download_url — ziglang.org
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    arch = _zig_arch(ctx)
    os   = _zig_os(ctx)
    ext  = "zip" if ctx.platform.os == "windows" else "tar.xz"
    asset = "zig-{}-{}-{}.{}".format(arch, os, version, ext)
    return "https://ziglang.org/download/{}/{}".format(version, asset)

# ---------------------------------------------------------------------------
# install_layout — strip top-level "zig-{arch}-{os}-{version}/" dir
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    arch  = _zig_arch(ctx)
    os    = _zig_os(ctx)
    exe   = "zig.exe" if ctx.platform.os == "windows" else "zig"
    strip = "zig-{}-{}-{}".format(arch, os, version)
    return {
        "type":             "archive",
        "strip_prefix":     strip,
        "executable_paths": [exe, "zig"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/zig"

def get_execute_path(ctx, _version):
    exe = "zig.exe" if ctx.platform.os == "windows" else "zig"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
