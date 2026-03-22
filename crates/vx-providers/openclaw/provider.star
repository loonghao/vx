# provider.star - openclaw provider
#
# OpenClaw: Open-source personal AI assistant / AI execution gateway
# ClawHub: The public skill registry for OpenClaw
#
# OpenClaw is an npm package. `vx openclaw` routes to `vx npm:openclaw`
# via RFC 0033 package_alias mechanism, just like vite, turbo, nx, etc.
#
# ClawHub CLI is bundled with the OpenClaw npm package.
# Requires Node.js >= 22.

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def",
     "system_permissions", "dep_def")
load("@vx//stdlib:http.star", "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "openclaw"
description = "OpenClaw - Open-source personal AI assistant with ClawHub skill registry"
homepage    = "https://openclaw.ai"
repository  = "https://github.com/openclaw/openclaw"
license     = "Apache-2.0"
ecosystem   = "nodejs"

# RFC 0033: route `vx openclaw` → `vx npm:openclaw`
package_alias = {"ecosystem": "npm", "package": "openclaw"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("openclaw",
        aliases = ["claw"],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "OpenClaw"},
        ],
    ),
    bundled_runtime_def("clawhub", bundled_with = "openclaw",
        description = "ClawHub - Skill registry CLI for OpenClaw",
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
    return fetch_json_versions(ctx, "https://registry.npmjs.org/openclaw", "npm_registry")

# ---------------------------------------------------------------------------
# download_url — not applicable (npm package)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/openclaw"

def get_execute_path(ctx, _version):
    exe = "openclaw.cmd" if ctx.platform.os == "windows" else "openclaw"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(_ctx, _version):
    return []

# ---------------------------------------------------------------------------
# deps — requires Node.js 22+
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("node", version = ">=22",
                reason = "OpenClaw requires Node.js 22 or later"),
    ]
