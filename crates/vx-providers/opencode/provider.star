# provider.star - opencode provider
#
# OpenCode — the open-source AI coding agent for the terminal.
#
# OpenCode is an npm package. `vx opencode` routes to `vx npm:opencode-ai` through
# the RFC 0033 package_alias mechanism. The package name differs from the binary
# name, so `executable` pins the binary to `opencode`.
#
# `vx install opencode` installs it globally and writes an `opencode` shim into
# ~/.vx/shims, which is what puts it on PATH so editors and agent platforms can
# discover it.

load("@vx//stdlib:provider.star",
     "runtime_def", "system_permissions", "dep_def")
load("@vx//stdlib:http.star", "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "opencode"
description = "OpenCode - the open source AI coding agent for the terminal"
homepage    = "https://opencode.ai"
repository  = "https://github.com/opencode-ai/opencode"
license     = "MIT"
ecosystem   = "nodejs"

# RFC 0033: route `vx opencode` -> `vx npm:opencode-ai` (binary: opencode)
package_alias = {"ecosystem": "npm", "package": "opencode-ai",
                 "executable": "opencode"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("opencode",
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
    return fetch_json_versions(ctx, "https://registry.npmjs.org/opencode-ai", "npm_registry")

# ---------------------------------------------------------------------------
# download_url — not applicable (npm package)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/opencode"

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
                reason = "OpenCode requires Node.js 18 or later"),
    ]
