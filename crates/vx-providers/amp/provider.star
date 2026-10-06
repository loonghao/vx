# provider.star - amp provider
#
# Amp — Sourcegraph's frontier coding agent and development environment.
#
# Amp is an npm package. `vx amp` routes to `vx npm:@ampcode/cli` through the
# RFC 0033 package_alias mechanism. The package is scoped, so `executable` pins the
# binary to `amp`. Upstream renamed the package from `@sourcegraph/amp` to
# `@ampcode/cli`; the latter is the maintained name.
#
# `vx install amp` installs it globally and writes an `amp` shim into ~/.vx/shims,
# which is what puts it on PATH so editors and agent platforms can discover it.

load("@vx//stdlib:provider.star",
     "runtime_def", "system_permissions", "dep_def")
load("@vx//stdlib:http.star", "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "amp"
description = "Amp - the frontier agent and development environment"
homepage    = "https://ampcode.com"
repository  = "https://ampcode.com"
license     = "Apache-2.0"
ecosystem   = "nodejs"

# RFC 0033: route `vx amp` -> `vx npm:@ampcode/cli` (binary: amp)
package_alias = {"ecosystem": "npm", "package": "@ampcode/cli",
                 "executable": "amp"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("amp",
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
    return fetch_json_versions(ctx, "https://registry.npmjs.org/@ampcode/cli", "npm_registry")

# ---------------------------------------------------------------------------
# download_url — not applicable (npm package)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/amp"

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
                reason = "Amp CLI requires Node.js 18 or later"),
    ]
