# provider.star - claude provider
#
# Claude Code — Anthropic's agentic coding CLI.
#
# Claude Code is an npm package. `vx claude` routes to `vx npm:@anthropic-ai/claude-code`
# through the RFC 0033 package_alias mechanism. The package is scoped, so `executable`
# pins the binary to `claude`; `claude-code` is kept as an alias because that is the
# name vx uses for this agent in `vx ai setup`.
#
# `vx install claude` installs it globally and writes a `claude` shim into ~/.vx/shims,
# which is what puts it on PATH so editors and agent platforms can discover it.

load("@vx//stdlib:provider.star",
     "runtime_def", "system_permissions", "dep_def")
load("@vx//stdlib:http.star", "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "claude"
description = "Claude Code - Anthropic's agentic coding command line tool"
homepage    = "https://claude.com/product/claude-code"
repository  = "https://github.com/anthropics/claude-code"
license     = "MIT"
ecosystem   = "nodejs"
aliases     = ["claude-code"]

# RFC 0033: route `vx claude` -> `vx npm:@anthropic-ai/claude-code` (binary: claude)
package_alias = {"ecosystem": "npm", "package": "@anthropic-ai/claude-code",
                 "executable": "claude"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("claude",
        aliases     = ["claude-code"],
        description = "Claude Code - agentic coding CLI",
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
        ctx, "https://registry.npmjs.org/@anthropic-ai/claude-code", "npm_registry")

# ---------------------------------------------------------------------------
# download_url — not applicable (npm package)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/claude"

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
        dep_def("node", version = ">=22",
                reason = "Claude Code requires Node.js 22 or later"),
    ]
