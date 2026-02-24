# provider.star - NASM provider
#
# Version source: https://www.nasm.us/pub/nasm/releasebuilds/
# NASM (Netwide Assembler) - portable 80x86 and x86-64 assembler
#
# Inheritance pattern: Level 1 (fully custom - uses nasm.us, not GitHub)
load("@vx//stdlib:github.star", "make_fetch_versions", "github_releases", "releases_to_versions")
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
    {
        "name":        "nasm",
        "executable":  "nasm",
        "description": "Netwide Assembler",
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "NASM version"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["www.nasm.us"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — nasm.us release directory listing
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch NASM versions from GitHub releases.

    NASM tags are like "nasm-2.16.03" or "2.16.03".
    We strip the "nasm-" prefix to get clean version numbers.
    """
    releases = github_releases(ctx, "netwide-assembler", "nasm", include_prereleases = False)
    return {
        "__type":           "github_versions",
        "source":           releases,
        "tag_key":          "tag_name",
        "strip_v_prefix":   False,
        "tag_prefix":       "nasm-",
        "skip_prereleases": True,
    }

# ---------------------------------------------------------------------------
# download_url — nasm.us official download
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build the NASM download URL from nasm.us.

    Args:
        ctx:     Provider context
        version: NASM version string, e.g. "2.16.03"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    os   = ctx.platform.os
    arch = ctx.platform.arch

    if os == "windows":
        if arch == "x64":
            # e.g. https://www.nasm.us/pub/nasm/releasebuilds/2.16.03/win64/nasm-2.16.03-win64.zip
            filename = "nasm-{}-win64.zip".format(version)
            return "https://www.nasm.us/pub/nasm/releasebuilds/{}/win64/{}".format(version, filename)
        else:
            filename = "nasm-{}-win32.zip".format(version)
            return "https://www.nasm.us/pub/nasm/releasebuilds/{}/win32/{}".format(version, filename)

    elif os == "macos":
        # macOS: use Homebrew or build from source
        # nasm.us provides macOS binaries in macosx/
        filename = "nasm-{}-macosx.zip".format(version)
        return "https://www.nasm.us/pub/nasm/releasebuilds/{}/macosx/{}".format(version, filename)

    elif os == "linux":
        # Linux: source tarball (most distros have nasm in package manager)
        filename = "nasm-{}.tar.xz".format(version)
        return "https://www.nasm.us/pub/nasm/releasebuilds/{}/{}".format(version, filename)

    return None

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx.platform.os

    if os == "windows":
        return {
            "type":             "archive",
            "strip_prefix":     "nasm-{}".format(version),
            "executable_paths": ["nasm.exe", "ndisasm.exe"],
        }
    elif os == "macos":
        return {
            "type":             "archive",
            "strip_prefix":     "nasm-{}".format(version),
            "executable_paths": ["nasm", "ndisasm"],
        }
    else:
        # Linux: source tarball, needs compilation
        return {
            "type":             "archive",
            "strip_prefix":     "nasm-{}".format(version),
            "executable_paths": ["nasm", "ndisasm"],
        }

# ---------------------------------------------------------------------------
# store_root — vx-managed install directory
# ---------------------------------------------------------------------------

def store_root(ctx, version):
    """Return the vx store root for this nasm version."""
    return ctx.paths.store_dir + "/nasm/" + version

# ---------------------------------------------------------------------------
# get_execute_path — resolve nasm executable
# ---------------------------------------------------------------------------

def get_execute_path(ctx, _version, install_dir):
    """Return the path to the nasm executable."""
    os  = ctx.platform.os
    exe = "nasm.exe" if os == "windows" else "nasm"
    return ctx.install_dir + "/" + exe

# ---------------------------------------------------------------------------
# post_install — set permissions on Unix
# ---------------------------------------------------------------------------

def post_install(ctx, _version):
    """Set execute permissions on Unix."""
    os = ctx.platform.os
    if os == "windows":
        return []
    return [
        {"type": "set_permissions", "path": ctx.install_dir + "/nasm",    "mode": "755"},
        {"type": "set_permissions", "path": ctx.install_dir + "/ndisasm", "mode": "755"},
    ]

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

# ---------------------------------------------------------------------------
# system_install — preferred on Linux
# ---------------------------------------------------------------------------

def system_install(ctx):
    os = ctx.platform.os
    if os == "windows":
        return {
            "strategies": [
                {"manager": "choco",  "package": "nasm", "priority": 80},
                {"manager": "winget", "package": "NASM.NASM", "priority": 70},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "nasm", "priority": 90},
            ],
        }
    elif os == "linux":
        return {
            "strategies": [
                {"manager": "apt", "package": "nasm", "priority": 80},
                {"manager": "dnf", "package": "nasm", "priority": 80},
            ],
        }
    return {}

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

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
