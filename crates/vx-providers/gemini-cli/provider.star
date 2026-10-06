# provider.star - gemini-cli provider
#
# Gemini CLI — Google's open-source AI agent for the terminal.
#
# Gemini CLI is an npm package. `vx gemini` routes to `vx npm:@google/gemini-cli`
# through the RFC 0033 package_alias mechanism. The package is scoped, so `executable`
# pins the binary to `gemini`; `gemini-cli` is kept as an alias because that is the
# name vx uses for this agent in `vx ai setup`.
#
# `vx install gemini` installs it globally and writes a `gemini` shim into ~/.vx/shims,
# which is what puts it on PATH so editors and agent platforms can discover it.

load("@vx//stdlib:provider.star",
     "runtime_def", "system_permissions", "dep_def")
load("@vx//stdlib:http.star", "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "gemini-cli"
description = "Gemini CLI - Google's open-source AI agent for the terminal"
homepage    = "https://geminicli.com"
repository  = "https://github.com/google-gemini/gemini-cli"
license     = "Apache-2.0"
ecosystem   = "nodejs"
aliases     = ["gemini"]

# RFC 0033: route `vx gemini` -> `vx npm:@google/gemini-cli` (binary: gemini)
package_alias = {"ecosystem": "npm", "package": "@google/gemini-cli",
                 "executable": "gemini"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("gemini",
        aliases     = ["gemini-cli"],
        description = "Gemini CLI - AI agent for the terminal",
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
    return fetch_json_versions(
        ctx, "https://registry.npmjs.org/@google/gemini-cli", "npm_registry")

# ---------------------------------------------------------------------------
# download_url — not applicable (npm package)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/gemini-cli"

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
        dep_def("node", version = ">=20",
                reason = "Gemini CLI requires Node.js 20 or later"),
    ]
