# provider.star - systemctl provider
#
# systemd system and service manager (Linux-only, system detection only)
# Inheritance pattern: Level 1 (fully custom, system-only, not installable)
#
# systemd is Linux-only and cannot be installed by vx.
# vx only detects the system installation.

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "systemctl"

def description():
    return "systemd system and service manager"

def homepage():
    return "https://systemd.io"

def repository():
    return "https://github.com/systemd/systemd"

def license():
    return "LGPL-2.1-or-later"

def ecosystem():
    return "system"

# ---------------------------------------------------------------------------
# Platform constraint: Linux-only
# ---------------------------------------------------------------------------

def supported_platforms():
    return [
        {"os": "linux", "arch": "x64"},
        {"os": "linux", "arch": "arm64"},
    ]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":             "systemctl",
        "executable":       "systemctl",
        "description":      "Control systemd services and units",
        "aliases":          [],
        "priority":         100,
        "auto_installable": False,
        "system_paths":     ["/usr/bin/systemctl", "/bin/systemctl"],
    },
    {
        "name":             "journalctl",
        "executable":       "journalctl",
        "description":      "View systemd journal logs",
        "bundled_with":     "systemctl",
        "auto_installable": False,
        "system_paths":     ["/usr/bin/journalctl", "/bin/journalctl"],
    },
    {
        "name":             "systemd-analyze",
        "executable":       "systemd-analyze",
        "description":      "Analyze systemd boot performance",
        "bundled_with":     "systemctl",
        "auto_installable": False,
        "system_paths":     ["/usr/bin/systemd-analyze", "/bin/systemd-analyze"],
    },
    {
        "name":             "loginctl",
        "executable":       "loginctl",
        "description":      "Control systemd login manager",
        "bundled_with":     "systemctl",
        "auto_installable": False,
        "system_paths":     ["/usr/bin/loginctl", "/bin/loginctl"],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": [],
    "fs":   ["/usr/bin", "/bin"],
    "exec": ["systemctl", "journalctl"],
}

# ---------------------------------------------------------------------------
# fetch_versions — system detection only
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Detect systemd version from system installation."""
    os = ctx["platform"]["os"]
    if os != "linux":
        return []

    result = ctx.get("execute", {})
    # Return a sentinel indicating system-only detection
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — not installable
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """systemd cannot be installed by vx — Linux system package only."""
    return None

# ---------------------------------------------------------------------------
# detect_system_installation
# ---------------------------------------------------------------------------

def detect_system_installation(ctx):
    """Detect systemd from system paths."""
    os = ctx["platform"]["os"]
    if os != "linux":
        return []

    results = []
    for path in ["/usr/bin/systemctl", "/bin/systemctl"]:
        if ctx["fs"]["exists"](path):
            results.append({
                "type":     "system_path",
                "path":     path,
                "priority": 100,
            })
    return results

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {}
