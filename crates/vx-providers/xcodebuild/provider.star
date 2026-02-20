# provider.star - xcodebuild provider
#
# Apple Xcode build tools (macOS-only, system detection only)
# Inheritance pattern: Level 1 (fully custom, system-only, not installable)
#
# Xcode tools are macOS-only and cannot be installed by vx.
# vx only detects the system installation.

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "xcodebuild"

def description():
    return "Apple Xcode build tools"

def homepage():
    return "https://developer.apple.com/xcode"

def license():
    return "Proprietary"

def ecosystem():
    return "system"

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
    },
    {
        "name":             "xcrun",
        "executable":       "xcrun",
        "description":      "Run or locate development tools",
        "bundled_with":     "xcodebuild",
        "auto_installable": False,
        "system_paths":     ["/usr/bin/xcrun"],
    },
    {
        "name":             "xcode-select",
        "executable":       "xcode-select",
        "description":      "Manage active Xcode installation",
        "bundled_with":     "xcodebuild",
        "auto_installable": False,
        "system_paths":     ["/usr/bin/xcode-select"],
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
    os = ctx["platform"]["os"]
    if os != "macos":
        return []
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — not installable
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Xcode cannot be installed by vx — install from Mac App Store or developer.apple.com."""
    return None

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for xcodebuild.

    xcodebuild is a macOS system tool; it is never installed by vx.
    """
    return "{vx_home}/store/xcodebuild"

def get_execute_path(ctx, version):
    """Return the executable path for xcodebuild (macOS-only)."""
    return "/usr/bin/xcodebuild"

def post_install(ctx, version, install_dir):
    """No post-install actions needed — system-only tool."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "DEVELOPER_DIR": "/Applications/Xcode.app/Contents/Developer",
    }
