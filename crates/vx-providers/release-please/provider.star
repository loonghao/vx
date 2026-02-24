# provider.star - release-please provider
#
# release-please is a Node.js tool run via `npx release-please`.
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star", "runtime_def", "dep_def", "github_permissions")
load("@vx//stdlib:http.star",     "fetch_json_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name          = "release-please"
description   = "Automated release PRs based on Conventional Commits"
homepage      = "https://github.com/googleapis/release-please"
repository    = "https://github.com/googleapis/release-please"
license       = "Apache-2.0"
ecosystem     = "nodejs"

# RFC 0033: route `vx release-please` → `vx npx release-please`
package_alias = {"ecosystem": "npm", "package": "release-please"}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("release-please",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(
    extra_hosts = ["registry.npmjs.org"],
    exec_cmds   = ["npx", "node"],
)

# ---------------------------------------------------------------------------
# fetch_versions — npm registry
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    return fetch_json_versions(
        ctx, "https://registry.npmjs.org/release-please", "npm_registry")

# ---------------------------------------------------------------------------
# download_url — npm package alias
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path queries
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/release-please"

def get_execute_path(ctx, _version):
    exe = "release-please.cmd" if ctx.platform.os == "windows" else "release-please"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(_ctx, _version):
    return []

# ---------------------------------------------------------------------------
# deps — requires node
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [dep_def("node", version = ">=18",
                    reason = "release-please is a Node.js tool run via npx")]
