# provider.star - Vite provider
#
# Vite is an npm package. `vx vite` routes to `vx npx vite`.
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star", "runtime_def", "dep_def", "system_permissions")
load("@vx//stdlib:http.star",     "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name          = "vite"
description   = "Vite - Next generation frontend build tool"
homepage      = "https://vitejs.dev"
repository    = "https://github.com/vitejs/vite"
license       = "MIT"
ecosystem     = "nodejs"

# RFC 0033: route `vx vite` → `vx npx vite`
package_alias = {"ecosystem": "npm", "package": "vite"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("vite",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "vite/\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(
    extra_hosts = ["registry.npmjs.org"],
    exec_cmds   = ["npx", "node"],
)

# ---------------------------------------------------------------------------
# fetch_versions — npm registry
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    return fetch_json_versions(ctx, "https://registry.npmjs.org/vite", "npm_registry")

# ---------------------------------------------------------------------------
# download_url — not applicable (npm package)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/vite"

def get_execute_path(ctx, _version):
    exe = "vite.cmd" if ctx.platform.os == "windows" else "vite"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(_ctx, _version):
    return []

# ---------------------------------------------------------------------------
# deps — Node.js version-dependent
# ---------------------------------------------------------------------------

def deps(_ctx, version):
    parts = version.split(".")
    major = int(parts[0]) if parts else 0
    if major >= 5:
        node_ver = ">=18"
    elif major >= 3:
        node_ver = ">=14.18"
    else:
        node_ver = ">=12"
    return [dep_def("node", version = node_ver,
                    reason = "Vite {} requires Node.js {}".format(version, node_ver))]
