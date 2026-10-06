# provider.star - qwen-code provider
#
# Qwen Code — Alibaba's AI-powered coding assistant for the terminal.
#
# Qwen Code is an npm package. `vx qwen` routes to `vx npm:@qwen-code/qwen-code`
# through the RFC 0033 package_alias mechanism. The package is scoped, so `executable`
# pins the binary to `qwen`; `qwen-code` is kept as an alias for symmetry with the
# package name.
#
# `vx install qwen` installs it globally and writes a `qwen` shim into ~/.vx/shims,
# which is what puts it on PATH so editors and agent platforms can discover it.

load("@vx//stdlib:provider.star",
     "runtime_def", "system_permissions", "dep_def")
load("@vx//stdlib:http.star", "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "qwen-code"
description = "Qwen Code - AI-powered coding assistant for the terminal"
homepage    = "https://github.com/QwenLM/qwen-code"
repository  = "https://github.com/QwenLM/qwen-code"
license     = "Apache-2.0"
ecosystem   = "nodejs"
aliases     = ["qwen"]

# RFC 0033: route `vx qwen` -> `vx npm:@qwen-code/qwen-code` (binary: qwen)
package_alias = {"ecosystem": "npm", "package": "@qwen-code/qwen-code",
                 "executable": "qwen"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("qwen",
        aliases     = ["qwen-code"],
        description = "Qwen Code - AI coding assistant CLI",
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
        ctx, "https://registry.npmjs.org/@qwen-code/qwen-code", "npm_registry")

# ---------------------------------------------------------------------------
# download_url — not applicable (npm package)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/qwen-code"

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
                reason = "Qwen Code requires Node.js 22 or later"),
    ]
