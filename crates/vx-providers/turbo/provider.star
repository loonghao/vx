# provider.star - Turborepo provider
#
# Turborepo is a high-performance build system for JavaScript/TypeScript monorepos.
# Written in Go (now Rust), it provides intelligent caching and task orchestration.
#
# Features:
# - Local and remote caching
# - Parallel task execution
# - Incremental builds
# - Content-aware hashing
#
# Usage:
#   vx turbo build           # Build with cache
#   vx turbo test --cache-dir=./cache  # Custom cache dir
#   vx turbo build --force   # Force rebuild

load("@vx//stdlib:provider.star", "runtime_def", "dep_def", "system_permissions")
load("@vx//stdlib:http.star",     "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name          = "turbo"
description   = "Turborepo - High-performance Build System for JavaScript/TypeScript"
homepage      = "https://turborepo.dev"
repository    = "https://github.com/vercel/turborepo"
license       = "MIT"
ecosystem     = "nodejs"
aliases       = ["turborepo"]

# RFC 0033: route `vx turbo` → `vx npx turbo`
package_alias = {"ecosystem": "npm", "package": "turbo"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("turbo",
        aliases = ["turborepo"],
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
    extra_hosts = ["registry.npmjs.org", "api.vercel.com"],
    exec_cmds   = ["npx", "node"],
)

# ---------------------------------------------------------------------------
# fetch_versions — npm registry
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    return fetch_json_versions(
        ctx, "https://registry.npmjs.org/turbo", "npm_registry")

# ---------------------------------------------------------------------------
# download_url — npm package alias
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/turbo"

def get_execute_path(ctx, _version):
    exe = "turbo.cmd" if ctx.platform.os == "windows" else "turbo"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return """
Turborepo installed successfully!

Quick Start:
  vx turbo build            # Build with cache
  vx turbo test             # Test with cache
  vx turbo lint             # Lint with cache
  vx turbo build test lint  # Run multiple tasks

Cache Commands:
  vx turbo build --force    # Force rebuild (skip cache)
  vx turbo prune --scope=web  # Prune for deployment

Configuration (turbo.json):
  {
    "$schema": "https://turbo.build/schema.json",
    "pipeline": {
      "build": {
        "outputs": [".next/**", "!.next/cache/**"]
      },
      "test": {
        "dependsOn": ["build"]
      }
    }
  }

Remote Cache:
  # Enable Vercel Remote Cache
  npx turbo login
  npx turbo link

Environment Variables:
  TURBO_CACHE_DIR=./cache   # Custom cache directory
  TURBO_TEAM=my-team        # Team for remote cache
  TURBO_TOKEN=xxx           # Token for remote cache
"""

def environment(_ctx, _version):
    return []

# ---------------------------------------------------------------------------
# deps — requires node
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    # Turborepo works best with Node.js 16+
    return [dep_def("node", version = ">=16",
                    reason = "Turborepo requires Node.js 16+")]
