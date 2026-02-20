# provider.star - Meson provider
#
# Meson is a Python-based build system distributed via PyPI.
# `vx meson` is routed to `vx uvx meson` (uv package alias pattern).
#
# Inheritance pattern: Level 3 (uv/pip package alias, no direct download)

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "meson"

def description():
    return "Meson - An extremely fast and user friendly build system"

def homepage():
    return "https://mesonbuild.com"

def repository():
    return "https://github.com/mesonbuild/meson"

def license():
    return "Apache-2.0"

def ecosystem():
    return "python"

def aliases():
    return ["mesonbuild"]

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
# Package alias configuration
# ---------------------------------------------------------------------------

package_alias = {
    "ecosystem": "uv",
    "package":   "meson",
}

# ---------------------------------------------------------------------------
# fetch_versions — PyPI
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch Meson versions from PyPI."""
    data = ctx["http"]["get_json"](
        "https://pypi.org/pypi/meson/json"
    )
    releases = data.get("releases", {})

    versions = []
    for v in releases.keys():
        prerelease = "a" in v or "b" in v or "rc" in v
        versions.append({
            "version":    v,
            "lts":        False,
            "prerelease": prerelease,
        })
    return versions

# ---------------------------------------------------------------------------
# download_url — not applicable (PyPI package)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    return None

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for meson."""
    return "{vx_home}/store/meson"

def get_execute_path(ctx, version):
    """Return the executable path for the given version (uv package alias)."""
    return "{install_dir}/meson"

def post_install(ctx, version, install_dir):
    """No post-install steps needed for meson."""
    return None

# ---------------------------------------------------------------------------
# deps — requires uv
# ---------------------------------------------------------------------------

def deps(ctx, version):
    return [
        {"runtime": "uv", "version": "*",
         "reason": "Meson is installed and run via uv/uvx"},
    ]
