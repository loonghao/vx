# provider.star - aider provider
#
# Aider — AI pair programming in your terminal.
#
# Aider is a Python package. `vx aider` routes to `vx uvx:aider-chat` through the
# RFC 0033 package_alias mechanism (same shape as `vx meson` -> `vx uvx:meson`), so
# each version runs in its own isolated Python environment. The distribution name
# (`aider-chat`) differs from the binary (`aider`), so `executable` pins the binary.
#
# `vx install aider` installs it globally and writes an `aider` shim into ~/.vx/shims,
# which is what puts it on PATH so editors and agent platforms can discover it.

load("@vx//stdlib:provider.star",
     "runtime_def", "system_permissions", "dep_def")
load("@vx//stdlib:http.star", "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "aider"
description = "Aider - AI pair programming in your terminal"
homepage    = "https://aider.chat"
repository  = "https://github.com/Aider-AI/aider"
license     = "Apache-2.0"
ecosystem   = "python"

# RFC 0033: route `vx aider` -> `vx uvx:aider-chat` (binary: aider)
package_alias = {"ecosystem": "uvx", "package": "aider-chat",
                 "executable": "aider"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("aider",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions — PyPI-based, no direct binary download
# ---------------------------------------------------------------------------

permissions = system_permissions(
    extra_hosts = ["pypi.org"],
    exec_cmds   = ["uvx", "uv"],
)

# ---------------------------------------------------------------------------
# fetch_versions — PyPI JSON API
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    return fetch_json_versions(
        ctx, "https://pypi.org/pypi/aider-chat/json", "pypi")

# ---------------------------------------------------------------------------
# download_url — not applicable (runs via uvx)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/aider"

# Execution goes through uvx, so there is no binary inside the provider store.
def get_execute_path(_ctx, _version):
    return None

def post_install(_ctx, _version):
    return None

def environment(_ctx, _version):
    return []

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("uv", reason = "Aider is installed and run via uv"),
    ]
