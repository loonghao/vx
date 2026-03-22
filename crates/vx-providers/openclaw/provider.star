# provider.star - openclaw provider
#
# OpenClaw: Open-source personal AI assistant / AI execution gateway
# ClawHub: The public skill registry for OpenClaw
#
# OpenClaw is installed via npm: `npm install -g openclaw@latest`
# ClawHub CLI is bundled with OpenClaw installation
#
# This provider manages the OpenClaw CLI and its bundled ClawHub commands.
# Requires Node.js >= 22.

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def",
     "system_permissions", "dep_def")
load("@vx//stdlib:http.star", "fetch_json_versions")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "openclaw"
description = "OpenClaw - Open-source personal AI assistant with ClawHub skill registry"
homepage    = "https://openclaw.ai"
repository  = "https://github.com/openclaw/openclaw"
license     = "Apache-2.0"
ecosystem   = "nodejs"

# Supported package prefixes for ecosystem:package syntax (RFC 0027)
# Enables `vx npm:openclaw` for installation
package_prefixes = ["npm", "npx"]

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
# Installation via npm global install
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    # OpenClaw versions are fetched from npm registry
    return fetch_json_versions(ctx, "https://registry.npmjs.org/openclaw", "npm_registry")

def download_url(_ctx, _version):
    # Not a direct binary download — installed via npm
    return None

def install_layout(_ctx, _version):
    return {
        "type": "npm_global",
        "package": "openclaw",
    }

# ---------------------------------------------------------------------------
# system_install — use npm global install
# ---------------------------------------------------------------------------

def system_install(_ctx, version):
    pkg = "openclaw@{}".format(version) if version != "latest" else "openclaw@latest"
    return {
        "strategies": [
            {"type": "npm_global", "package": pkg,
             "command": "npm install -g {}".format(pkg)},
        ],
    }

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

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return [
        dep_def("node", version = ">=22",
                reason = "OpenClaw requires Node.js 22 or later"),
    ]
