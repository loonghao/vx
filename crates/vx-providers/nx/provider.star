# provider.star - Nx provider
#
# Nx is a smart build system for monorepos with powerful caching.
# It caches build artifacts and test results, supporting remote cache.
#
# Features:
# - Local and remote caching
# - Affected project detection
# - Task orchestration
# - Code generation
#
# Usage:
#   vx nx build myapp        # Build with cache
#   vx nx test myapp --skip-nx-cache  # Skip cache
#   vx nx run-many -t build  # Build all projects

load("@vx//stdlib:provider.star", "runtime_def", "dep_def", "system_permissions")
load("@vx//stdlib:http.star",     "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name          = "nx"
description   = "Nx - Smart, Fast and Extensible Build System for Monorepos"
homepage      = "https://nx.dev"
repository    = "https://github.com/nrwl/nx"
license       = "MIT"
ecosystem     = "nodejs"
aliases       = ["nrwl"]

# RFC 0033: route `vx nx` → `vx npx nx`
package_alias = {"ecosystem": "npm", "package": "nx"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("nx",
        aliases = ["nrwl"],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(
    extra_hosts = ["registry.npmjs.org", "cloud.nx.app"],
    exec_cmds   = ["npx", "node"],
)

# ---------------------------------------------------------------------------
# fetch_versions — npm registry
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    return fetch_json_versions(
        ctx, "https://registry.npmjs.org/nx", "npm_registry")

# ---------------------------------------------------------------------------
# download_url — npm package alias
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/nx"

def get_execute_path(ctx, _version):
    exe = "nx.cmd" if ctx.platform.os == "windows" else "nx"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return """
Nx installed successfully!

Quick Start:
  vx nx init                # Initialize Nx in existing project
  vx nx build myapp         # Build with cache
  vx nx test myapp          # Test with cache
  vx nx affected -t build   # Build affected projects only

Cache Commands:
  vx nx reset               # Clear cache and daemon
  vx nx daemon --stop       # Stop daemon

Configuration (nx.json):
  {
    "tasksRunnerOptions": {
      "default": {
        "runner": "nx/tasks-runners/default",
        "options": {
          "cacheableOperations": ["build", "test", "lint"]
        }
      }
    }
  }

Remote Cache:
  # Enable Nx Cloud for remote caching
  nx connect

Environment Variables:
  NX_CACHE_DIRECTORY=.nx-cache   # Custom cache directory
  NX_DAEMON=false                # Disable daemon
"""

def environment(_ctx, _version):
    return []

# ---------------------------------------------------------------------------
# deps — requires node
# ---------------------------------------------------------------------------

def deps(_ctx, version):
    parts = version.split(".")
    major = int(parts[0]) if parts else 0
    # Nx 18+ requires Node.js 18+
    if major >= 18:
        node_ver = ">=18"
    elif major >= 16:
        node_ver = ">=16"
    else:
        node_ver = ">=14"
    return [dep_def("node", version = node_ver,
                    reason = "Nx {} requires Node.js {}".format(version, node_ver))]
