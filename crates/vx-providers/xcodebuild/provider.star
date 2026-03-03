# provider.star - xcodebuild provider
#
# Apple Xcode build tools (macOS-only, system detection only)
# Inheritance pattern: Level 1 (fully custom, system-only, not installable)
#
# Xcode tools are macOS-only and cannot be installed by vx.
# vx only detects the system installation.

load("@vx//stdlib:env.star", "env_set")
load("@vx//stdlib:provider.star", "runtime_def", "bundled_runtime_def", "system_permissions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "xcodebuild"
description = "Apple Xcode build tools"
homepage    = "https://developer.apple.com/xcode"
license     = "Proprietary"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Platform constraint: macOS-only
# ---------------------------------------------------------------------------

platforms = {"os": ["macos"]}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("xcodebuild",
        description = "Build Xcode projects and workspaces",
    ),
    bundled_runtime_def("xcrun", "xcodebuild",
        description = "Run or locate development tools",
    ),
    bundled_runtime_def("xcode-select", "xcodebuild",
        description = "Manage active Xcode installation",
    ),
    bundled_runtime_def("swift", "xcodebuild",
        description = "Swift programming language compiler",
    ),
    bundled_runtime_def("swiftc", "xcodebuild",
        description = "Swift compiler",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(
    exec_cmds = ["xcodebuild", "xcrun", "xcode-select"],
)

# ---------------------------------------------------------------------------
# fetch_versions — system detection only
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Detect Xcode version from system installation."""
    os = ctx.platform.os
    if os != "macos":
        return []
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — not installable
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    """Xcode cannot be installed by vx — install from Mac App Store or developer.apple.com."""
    return None

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for xcodebuild.

    xcodebuild is a macOS system tool; it is never installed by vx.
    """
    return ctx.vx_home + "/store/xcodebuild"

def get_execute_path(_ctx, _version):
    """Return the executable path for xcodebuild (macOS-only)."""
    return "/usr/bin/xcodebuild"

def post_install(_ctx, _version):
    """No post-install actions needed — system-only tool."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(_ctx, _version):
    return [
        env_set("DEVELOPER_DIR", "/Applications/Xcode.app/Contents/Developer"),
    ]
