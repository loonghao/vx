# provider.star - rez provider
#
# Rez: Cross-platform package manager for deterministic environments
# Executed via `uvx rez` — vx rez == vx uvx:rez (RFC 0033 package_alias routing)
# Bundled runtimes: rez-env, rez-build (available when rez is pip-installed)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def",
     "dep_def", "system_permissions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "rez"
description = "Cross-platform package manager for deterministic environments"
homepage    = "https://rez.readthedocs.io"
repository  = "https://github.com/AcademySoftwareFoundation/rez"
license     = "Apache-2.0"
ecosystem   = "python"

# RFC 0033: route `vx rez` → `vx uvx:rez`
package_alias = {"ecosystem": "uvx", "package": "rez"}

# Supported package prefixes for ecosystem:package syntax (RFC 0027)
# Enables `vx rez:<package>` for VFX package installation
package_prefixes = ["rez"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("rez",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+"},
        ],
    ),
    bundled_runtime_def("rez-env",   bundled_with = "rez"),
    bundled_runtime_def("rez-build", bundled_with = "rez"),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(
    extra_hosts = ["pypi.org"],
    exec_cmds   = ["uvx", "uv"],
)

# ---------------------------------------------------------------------------
# download_url — not applicable (runs via uvx)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/rez"

def get_execute_path(_ctx, _version):
    return None

def post_install(_ctx, _version):
    return None

def environment(_ctx, _version):
    return []

# ---------------------------------------------------------------------------
# deps — requires uv
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("uv", reason = "Rez is installed and run via uv/uvx"),
    ]
