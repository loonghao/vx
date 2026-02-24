# provider.star - NASM provider
#
# NASM (Netwide Assembler) - portable 80x86 and x86-64 assembler
# Version source: nasm.us (tags: "nasm-2.16.03")
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "fetch_versions_with_tag_prefix",
     "system_permissions",
     "post_extract_permissions",
     "cross_platform_install")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "nasm"
description = "NASM - Netwide Assembler, portable 80x86 and x86-64 assembler"
homepage    = "https://www.nasm.us"
repository  = "https://github.com/netwide-assembler/nasm"
license     = "BSD-2-Clause"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("nasm",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "NASM version"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(
    extra_hosts = ["www.nasm.us"],
)

# ---------------------------------------------------------------------------
# fetch_versions — tags like "nasm-2.16.03"
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix(
    "netwide-assembler", "nasm", tag_prefix = "nasm-")

# ---------------------------------------------------------------------------
# download_url — nasm.us official download
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    if os == "windows":
        if arch == "x64":
            filename = "nasm-{}-win64.zip".format(version)
            return "https://www.nasm.us/pub/nasm/releasebuilds/{}/win64/{}".format(version, filename)
        else:
            filename = "nasm-{}-win32.zip".format(version)
            return "https://www.nasm.us/pub/nasm/releasebuilds/{}/win32/{}".format(version, filename)
    elif os == "macos":
        filename = "nasm-{}-macosx.zip".format(version)
        return "https://www.nasm.us/pub/nasm/releasebuilds/{}/macosx/{}".format(version, filename)
    elif os == "linux":
        filename = "nasm-{}.tar.xz".format(version)
        return "https://www.nasm.us/pub/nasm/releasebuilds/{}/{}".format(version, filename)
    return None

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    exe_paths = ["nasm.exe", "ndisasm.exe"] if ctx.platform.os == "windows" else ["nasm", "ndisasm"]
    return {
        "type":             "archive",
        "strip_prefix":     "nasm-{}".format(version),
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# post_extract — set +x on Unix
# ---------------------------------------------------------------------------

post_extract = post_extract_permissions(["nasm", "ndisasm"])

# ---------------------------------------------------------------------------
# system_install
# ---------------------------------------------------------------------------

system_install = cross_platform_install(
    windows = "NASM.NASM",
    macos   = "nasm",
    linux   = "nasm",
    windows_priority = 80,
)

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/nasm"

def get_execute_path(ctx, _version):
    exe = "nasm.exe" if ctx.platform.os == "windows" else "nasm"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []

# ---------------------------------------------------------------------------
# uninstall — vx-managed versions use default dir removal;
#             system-installed versions delegate to package manager
# ---------------------------------------------------------------------------

def uninstall(ctx, version):
    """Uninstall NASM.

    vx-managed versions (store directory exists): return False to let vx
    remove the store directory.
    system version: delegate to the system package manager.
    """
    if version != "system":
        # vx-managed install — let default directory removal handle it
        return False

    os = ctx.platform.os
    if os == "windows":
        return {
            "type": "system_uninstall",
            "strategies": [
                {"manager": "choco",  "package": "nasm",      "priority": 80},
                {"manager": "winget", "package": "NASM.NASM", "priority": 70},
            ],
        }
    elif os == "macos":
        return {
            "type": "system_uninstall",
            "strategies": [
                {"manager": "brew", "package": "nasm", "priority": 90},
            ],
        }
    elif os == "linux":
        return {
            "type": "system_uninstall",
            "strategies": [
                {"manager": "apt", "package": "nasm", "priority": 80},
                {"manager": "dnf", "package": "nasm", "priority": 80},
            ],
        }
    return False
