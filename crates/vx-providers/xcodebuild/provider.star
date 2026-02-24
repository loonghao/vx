# provider.star - xcodebuild provider
#
# Apple Xcode build tools (macOS-only, system detection only)
# Inheritance pattern: Level 1 (fully custom, system-only, not installable)
#
# Xcode tools are macOS-only and cannot be installed by vx.
# vx only detects the system installation.

load("@vx//stdlib:env.star", "env_set")

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

def supported_platforms():
    return [
        {"os": "macos", "arch": "x64"},
        {"os": "macos", "arch": "arm64"},
    ]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":             "xcodebuild",
        "executable":       "xcodebuild",
        "description":      "Build Xcode projects and workspaces",
        "aliases":          [],
        "priority":         100,
        "auto_installable": False,
        "system_paths":     [
            "/usr/bin/xcodebuild",
            "/Applications/Xcode.app/Contents/Developer/usr/bin/xcodebuild",
        ],
        "test_commands": [
            {"command": "{executable} -version", "name": "version_check", "expected_output": "Xcode \\d+"},
        ],
    },
    {
        "name":             "xcrun",
        "executable":       "xcrun",
        "description":      "Run or locate development tools",
        "bundled_with":     "xcodebuild",
        "auto_installable": False,
        "system_paths":     ["/usr/bin/xcrun"],
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":             "xcode-select",
        "executable":       "xcode-select",
        "description":      "Manage active Xcode installation",
        "bundled_with":     "xcodebuild",
        "auto_installable": False,
        "system_paths":     ["/usr/bin/xcode-select"],
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":             "swift",
        "executable":       "swift",
        "description":      "Swift programming language compiler",
        "bundled_with":     "xcodebuild",
        "auto_installable": False,
        "system_paths":     [
            "/usr/bin/swift",
            "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/swift",
        ],
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":             "swiftc",
        "executable":       "swiftc",
        "description":      "Swift compiler",
        "bundled_with":     "xcodebuild",
        "auto_installable": False,
        "system_paths":     [
            "/usr/bin/swiftc",
            "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/swiftc",
        ],
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": [],
    "fs":   [
        "/usr/bin",
        "/Applications/Xcode.app",
    ],
    "exec": ["xcodebuild", "xcrun", "xcode-select"],
}

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
