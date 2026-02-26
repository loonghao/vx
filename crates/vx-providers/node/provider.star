# provider.star - Node.js provider
#
# Version source: https://nodejs.org/dist/index.json (official API, no rate limiting)
# Bundled runtimes: npm, npx (included in every Node.js release)
# Archive layout: node-v{version}-{os}-{arch}/ with bin/ subdir on Unix
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def",
     "fetch_versions_from_api",
     "system_permissions",
     "bin_subdir_env", "bin_subdir_execute_path",
     "post_extract_permissions", "pre_run_ensure_deps")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "node"
description = "Node.js - JavaScript runtime built on Chrome's V8 engine"
homepage    = "https://nodejs.org"
repository  = "https://github.com/nodejs/node"
license     = "MIT"
ecosystem   = "nodejs"
aliases     = ["nodejs"]

# Supported package prefixes for ecosystem:package syntax (RFC 0027)
# Enables `vx npm:<package>` and `vx npx:<package>` for Node.js package execution
package_prefixes = ["npm", "npx"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("node",
        aliases = ["nodejs"],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "^v?\\d+\\.\\d+\\.\\d+"},
            {"command": "{executable} -e \"console.log('ok')\"", "name": "eval_check",
             "expected_output": "ok"},
        ],
    ),
    bundled_runtime_def("npm",  bundled_with = "node",
        version_pattern = "^\\d+\\.\\d+\\.\\d+"),
    bundled_runtime_def("npx",  bundled_with = "node",
        version_pattern = "^\\d+\\.\\d+\\.\\d+"),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(extra_hosts = ["nodejs.org"])

# ---------------------------------------------------------------------------
# fetch_versions — nodejs.org official API (no rate limiting)
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_api(
    "https://nodejs.org/dist/index.json",
    "nodejs_org",
)

# ---------------------------------------------------------------------------
# Platform helpers
# Node.js uses: win/darwin/linux × x64/x86/arm64/armv7l
# ---------------------------------------------------------------------------

_NODE_PLATFORMS = {
    "windows/x64":  ("win",    "x64"),
    "windows/x86":  ("win",    "x86"),
    "macos/x64":    ("darwin", "x64"),
    "macos/arm64":  ("darwin", "arm64"),
    "linux/x64":    ("linux",  "x64"),
    "linux/arm64":  ("linux",  "arm64"),
    "linux/armv7":  ("linux",  "armv7l"),
}

def _node_platform(ctx):
    return _NODE_PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url — nodejs.org
# Windows: node-v{version}-win-{arch}.zip
# Unix:    node-v{version}-{os}-{arch}.tar.xz
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _node_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform[0], platform[1]
    if ctx.platform.os == "windows":
        filename = "node-v{}-{}-{}.zip".format(version, os_str, arch_str)
    else:
        filename = "node-v{}-{}-{}.tar.xz".format(version, os_str, arch_str)
    return "https://nodejs.org/dist/v{}/{}".format(version, filename)

# ---------------------------------------------------------------------------
# install_layout — strip top-level "node-v{version}-{os}-{arch}/" dir
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    platform = _node_platform(ctx)
    if not platform:
        return {"type": "archive", "strip_prefix": "", "executable_paths": ["node"]}
    os_str, arch_str = platform[0], platform[1]
    strip = "node-v{}-{}-{}".format(version, os_str, arch_str)
    if ctx.platform.os == "windows":
        exe_paths = ["node.exe", "npm.cmd", "npx.cmd"]
    else:
        exe_paths = ["bin/node", "bin/npm", "bin/npx"]
    return {
        "type":             "archive",
        "strip_prefix":     strip,
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# post_extract — ensure bundled tools have execute permissions on Unix
# ---------------------------------------------------------------------------

post_extract = post_extract_permissions(
    ["bin/node", "bin/npm", "bin/npx", "bin/corepack"],
)

# ---------------------------------------------------------------------------
# pre_run — ensure node_modules before `npm run` / `npm run-script`
# ---------------------------------------------------------------------------

pre_run = pre_run_ensure_deps("npm",
    trigger_args = ["run", "run-script"],
    check_file   = "package.json",
    install_dir  = "node_modules",
)

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/node"

get_execute_path = bin_subdir_execute_path("node")
environment      = bin_subdir_env()

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
