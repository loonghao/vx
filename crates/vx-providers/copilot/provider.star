# provider.star - copilot provider
#
# GitHub Copilot CLI — brings the Copilot coding agent to the terminal.
#
# Copilot CLI is an npm package. `vx copilot` routes to `vx npm:@github/copilot`
# through the RFC 0033 package_alias mechanism. The package is scoped, so `executable`
# pins the binary to `copilot`. Note the unscoped `copilot` package on npm is a
# placeholder that only tells you to install `@github/copilot`.
#
# `vx install copilot` installs it globally and writes a `copilot` shim into
# ~/.vx/shims, which is what puts it on PATH so editors and agent platforms can
# discover it.

load("@vx//stdlib:provider.star",
     "runtime_def", "system_permissions", "dep_def")
load("@vx//stdlib:http.star", "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "copilot"
description = "GitHub Copilot CLI - the Copilot coding agent in your terminal"
homepage    = "https://github.com/github/copilot-cli"
repository  = "https://github.com/github/copilot-cli"
license     = "MIT"
ecosystem   = "nodejs"

# RFC 0033: route `vx copilot` -> `vx npm:@github/copilot` (binary: copilot)
package_alias = {"ecosystem": "npm", "package": "@github/copilot",
                 "executable": "copilot"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("copilot",
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
    return fetch_json_versions(ctx, "https://registry.npmjs.org/@github/copilot", "npm_registry")

# ---------------------------------------------------------------------------
# download_url — not applicable (npm package)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/copilot"

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
                reason = "GitHub Copilot CLI requires Node.js 18 or later"),
    ]
