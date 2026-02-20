# provider.star - Visual Studio Code provider
#
# Version source: https://update.code.visualstudio.com/api/releases/stable
#
# VS Code provides portable archives for all platforms.
# Inheritance pattern: Level 1 (fully custom - uses VS Code update API)

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "vscode"

def description():
    return "Visual Studio Code - Code editing. Redefined."

def homepage():
    return "https://code.visualstudio.com"

def repository():
    return "https://github.com/microsoft/vscode"

def license():
    return "MIT"

def ecosystem():
    return "devtools"

def aliases():
    return ["code", "vs-code"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "code",
        "executable":  "code",
        "description": "Visual Studio Code editor",
        "aliases":     ["vscode", "vs-code"],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["update.code.visualstudio.com", "az764295.vo.msecnd.net"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — VS Code update API
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch VS Code stable versions from the official update API."""
    releases = ctx["http"]["get_json"](
        "https://update.code.visualstudio.com/api/releases/stable"
    )

    versions = []
    for v in releases:
        versions.append({"version": v, "lts": True, "prerelease": False})

    return versions

# ---------------------------------------------------------------------------
# download_url — VS Code portable archive
# ---------------------------------------------------------------------------

def _vscode_platform(ctx):
    """Map vx platform to VS Code platform/archive strings."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    # (platform_id, ext)
    platforms = {
        "windows/x64":   ("win32-x64-archive",   "zip"),
        "windows/arm64": ("win32-arm64-archive",  "zip"),
        "macos/x64":     ("darwin",               "zip"),
        "macos/arm64":   ("darwin-arm64",         "zip"),
        "linux/x64":     ("linux-x64",            "tar.gz"),
        "linux/arm64":   ("linux-arm64",          "tar.gz"),
        "linux/armv7":   ("linux-armhf",          "tar.gz"),
    }
    return platforms.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the VS Code portable archive download URL.

    Args:
        ctx:     Provider context
        version: VS Code version string, e.g. "1.86.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _vscode_platform(ctx)
    if not platform:
        return None

    platform_id, ext = platform[0], platform[1]

    # https://update.code.visualstudio.com/1.86.0/linux-x64/stable
    # redirects to the actual CDN URL
    return "https://update.code.visualstudio.com/{}/{}/stable".format(
        version, platform_id
    )

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx["platform"]["os"]

    if os == "windows":
        exe_paths = ["bin/code.cmd", "Code.exe"]
    elif os == "macos":
        exe_paths = ["Visual Studio Code.app/Contents/Resources/app/bin/code"]
    else:
        exe_paths = ["bin/code"]

    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    os = ctx["platform"]["os"]
    if os == "windows":
        return {"PATH": install_dir + "/bin"}
    elif os == "macos":
        return {"PATH": install_dir + "/Visual Studio Code.app/Contents/Resources/app/bin"}
    else:
        return {"PATH": install_dir + "/bin"}

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for vscode."""
    return "{vx_home}/store/vscode"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    if os == "windows":
        return "{install_dir}/bin/code.cmd"
    elif os == "macos":
        return "{install_dir}/Visual Studio Code.app/Contents/Resources/app/bin/code"
    else:
        return "{install_dir}/bin/code"

def post_install(ctx, version, install_dir):
    """No post-install actions needed for vscode."""
    return None

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    return []
