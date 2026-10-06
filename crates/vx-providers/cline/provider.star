# provider.star - cline provider
#
# Cline — autonomous coding agent CLI capable of creating and editing files and
# running commands.
#
# Cline is an npm package. `vx cline` routes to `vx npm:cline` through the RFC 0033
# package_alias mechanism. Package and binary share the name `cline`, so no
# `executable` override is needed here.
#
# `vx install cline` installs it globally and writes a `cline` shim into ~/.vx/shims,
# which is what puts it on PATH so editors and agent platforms can discover it.

load("@vx//stdlib:provider.star",
     "runtime_def", "system_permissions", "dep_def")
load("@vx//stdlib:http.star", "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "cline"
description = "Cline - autonomous coding agent for your terminal and editor"
homepage    = "https://cline.bot"
repository  = "https://github.com/cline/cline"
license     = "Apache-2.0"
ecosystem   = "nodejs"

# RFC 0033: route `vx cline` -> `vx npm:cline` (binary name matches the package)
package_alias = {"ecosystem": "npm", "package": "cline"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("cline",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions — npm-based, no direct binary download
# ---------------------------------------------------------------------------

permissions = system_permissions(
    extra_hosts = ["registry.npmjs.org"],
    exec_cmds   = ["npm", "npx", "node"],
)

# ---------------------------------------------------------------------------
# fetch_versions — npm registry
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    return fetch_json_versions(ctx, "https://registry.npmjs.org/cline", "npm_registry")

# ---------------------------------------------------------------------------
# download_url — not applicable (npm package)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/cline"

# Execution goes through the package manager, so there is no binary inside the
# provider store to point at.
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
        dep_def("node", version = ">=18",
                reason = "Cline CLI requires Node.js 18 or later"),
    ]
