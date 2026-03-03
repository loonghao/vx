# provider.star - systemctl provider
#
# systemd system and service manager (Linux-only, system detection only)
# Inheritance pattern: Level 1 (fully custom, system-only, not installable)
#
# systemd is Linux-only and cannot be installed by vx.
# vx only detects the system installation.

load("@vx//stdlib:provider.star", "runtime_def", "bundled_runtime_def", "system_permissions")

description = "systemd system and service manager"
homepage    = "https://systemd.io"
repository  = "https://github.com/systemd/systemd"
license     = "LGPL-2.1-or-later"
ecosystem   = "system"

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
    runtime_def("systemctl",
        description = "Control systemd services and units",
    ),
    bundled_runtime_def("journalctl", "systemctl",
        description = "View systemd journal logs",
    ),
    bundled_runtime_def("systemd-analyze", "systemctl",
        description = "Analyze systemd boot performance",
    ),
    bundled_runtime_def("loginctl", "systemctl",
        description = "Control systemd login manager",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(
    exec_cmds = ["systemctl", "journalctl"],
)

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "systemctl"

# ---------------------------------------------------------------------------
# fetch_versions — system detection only
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Detect systemd version from system installation."""
    os = ctx.platform.os
    if os != "linux":
        return []

    # Return a sentinel indicating system-only detection
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — not installable
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    """systemd cannot be installed by vx — Linux system package only."""
    return None

# ---------------------------------------------------------------------------
# detect_system_installation
# ---------------------------------------------------------------------------

def detect_system_installation(ctx):
    """Detect systemd from system paths."""
    os = ctx.platform.os
    if os != "linux":
        return []

    results = []
    for path in ["/usr/bin/systemctl", "/bin/systemctl"]:
        if ctx.fs.exists(path):
            results.append({
                "type":     "system_path",
                "path":     path,
                "priority": 100,
            })
    return results

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for systemctl.

    systemctl is a system-only tool; it is never installed by vx.
    """
    return ctx.vx_home + "/store/systemctl"

def get_execute_path(_ctx, _version):
    """Return the executable path for systemctl.

    Always resolved from system paths on Linux.
    """
    return "/usr/bin/systemctl"

def post_install(_ctx, _version):
    """No post-install actions needed — system-only tool."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(_ctx, _version):
    return []
