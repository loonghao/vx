# provider.star - Meson provider
#
# Meson is a Python-based build system distributed via PyPI.
# Executed via `uvx meson` — each version runs in its own isolated Python env.
#
# vx meson  ==  vx uvx meson  (RFC 0033 package_alias routing)

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "meson"
description = "Meson - An extremely fast and user friendly build system"
homepage    = "https://mesonbuild.com"
repository  = "https://github.com/mesonbuild/meson"
license     = "Apache-2.0"
ecosystem   = "python"
aliases     = ["mesonbuild"]

# RFC 0033: route `vx meson` → `vx uvx:meson`
# This means meson runs in an isolated uv-managed Python environment,
# with proper version pinning per project.
package_alias = {"ecosystem": "uvx", "package": "meson"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "meson",
        "executable":  "meson",
        "description": "Meson build system",
        "aliases":     ["mesonbuild"],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "^\\d+\\.\\d+"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["pypi.org"],
    "fs":   [],
    "exec": ["uvx", "uv"],
}

# ---------------------------------------------------------------------------
# download_url — not applicable (runs via uvx)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for meson."""
    return ctx.vx_home + "/store/meson"

def get_execute_path(_ctx, version):
    """Return the executable path for the given version (pip package)."""
    return None

def post_install(_ctx, _version):
    """No post-install steps needed for meson."""
    return None

# ---------------------------------------------------------------------------
# deps — requires uv
# ---------------------------------------------------------------------------

def deps(_ctx, version):
    return [
        {"runtime": "uv", "version": "*",
         "reason": "Meson is installed and run via uv pip"},
    ]
