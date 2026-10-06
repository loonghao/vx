# provider.star - codex provider
#
# OpenAI Codex CLI — a coding agent that runs locally and edits your working tree.
#
# Codex is an npm package. `vx codex` routes to `vx npm:@openai/codex` through the
# RFC 0033 package_alias mechanism. The package is scoped, so `executable` pins the
# binary to `codex` — without it vx would look for a binary named "@openai/codex".
#
# `vx install codex` installs it globally and writes a `codex` shim into ~/.vx/shims,
# which is what puts it on PATH so editors and agent platforms can discover it.

load("@vx//stdlib:provider.star",
     "runtime_def", "system_permissions", "dep_def")
load("@vx//stdlib:http.star", "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "codex"
description = "OpenAI Codex CLI - coding agent that runs on your machine"
homepage    = "https://developers.openai.com/codex/cli"
repository  = "https://github.com/openai/codex"
license     = "Apache-2.0"
ecosystem   = "nodejs"

# RFC 0033: route `vx codex` -> `vx npm:@openai/codex` (binary: codex)
package_alias = {"ecosystem": "npm", "package": "@openai/codex",
                 "executable": "codex"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("codex",
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
    return fetch_json_versions(ctx, "https://registry.npmjs.org/@openai/codex", "npm_registry")

# ---------------------------------------------------------------------------
# download_url — not applicable (npm package)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/codex"

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
        dep_def("node", version = ">=16",
                reason = "Codex CLI requires Node.js 16 or later"),
    ]
