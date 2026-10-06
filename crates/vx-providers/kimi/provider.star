# provider.star - kimi provider
#
# Kimi Code — Moonshot AI's agentic coding CLI.
#
# Kimi Code is an npm package. `vx kimi` routes to `vx npm:@moonshot-ai/kimi-code`
# through the RFC 0033 package_alias mechanism. The package is scoped, so `executable`
# pins the binary to `kimi`.
#
# Note: the older PyPI package `kimi-cli` is archived upstream; `@moonshot-ai/kimi-code`
# is the maintained distribution.
#
# `vx install kimi` installs it globally and writes a `kimi` shim into ~/.vx/shims,
# which is what puts it on PATH so editors and agent platforms can discover it.

load("@vx//stdlib:provider.star",
     "runtime_def", "system_permissions", "dep_def")
load("@vx//stdlib:http.star", "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "kimi"
description = "Kimi Code - Moonshot AI's agentic coding command line tool"
homepage    = "https://www.kimi.com"
repository  = "https://github.com/MoonshotAI/kimi-code"
license     = "Apache-2.0"
ecosystem   = "nodejs"

# RFC 0033: route `vx kimi` -> `vx npm:@moonshot-ai/kimi-code` (binary: kimi)
package_alias = {"ecosystem": "npm", "package": "@moonshot-ai/kimi-code",
                 "executable": "kimi"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("kimi",
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
        ctx, "https://registry.npmjs.org/@moonshot-ai/kimi-code", "npm_registry")

# ---------------------------------------------------------------------------
# download_url — not applicable (npm package)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/kimi"

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
                reason = "Kimi Code requires Node.js 22 or later"),
    ]
