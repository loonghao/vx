# provider.star - Node.js provider
#
# Node.js - JavaScript runtime built on Chrome's V8 JavaScript engine
# Downloads from nodejs.org/dist
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def",
     "github_permissions",
     "bin_subdir_env", "bin_subdir_execute_path",
     "post_extract_permissions", "pre_run_ensure_deps",
     "fetch_versions_from_api", "path_fns")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "node"
description = "Node.js - JavaScript runtime built on Chrome's V8 JavaScript engine"
homepage    = "https://nodejs.org"
repository  = "https://github.com/nodejs/node"
license     = "MIT"
ecosystem   = "nodejs"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("node",
        aliases         = ["nodejs"],
        version_pattern = "v\\d+\\.\\d+\\.\\d+",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "v\\d+\\.\\d+"},
        ],
    ),
    bundled_runtime_def("npm", "node",
        description     = "Node Package Manager",
        version_pattern = "\\d+\\.\\d+\\.\\d+",
    ),
    bundled_runtime_def("npx", "node",
        description     = "Node Package eXecute",
        version_pattern = "\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["nodejs.org"])

# ---------------------------------------------------------------------------
# fetch_versions — from nodejs.org official API
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_api(
    "https://nodejs.org/dist/index.json",
    "nodejs_org",
)

# ---------------------------------------------------------------------------
# Platform helpers
# Node uses: node-v{version}-{os}-{arch}.{ext}
# ---------------------------------------------------------------------------

_NODE_PLATFORMS = {
    "windows/x64":   ("win",    "x64"),
    "windows/x86":   ("win",    "x86"),
    "windows/arm64": ("win",    "arm64"),
    "macos/x64":     ("darwin", "x64"),
    "macos/arm64":   ("darwin", "arm64"),
    "linux/x64":     ("linux",  "x64"),
    "linux/arm64":   ("linux",  "arm64"),
    "linux/armv7":   ("linux",  "armv7l"),
    "linux/ppc64":   ("linux",  "ppc64le"),
    "linux/s390x":   ("linux",  "s390x"),
}

def _node_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _NODE_PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _node_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform[0], platform[1]
    # macOS uses .tar.gz, Linux uses .tar.xz, Windows uses .zip
    if ctx.platform.os == "windows":
        ext = "zip"
    elif ctx.platform.os == "macos":
        ext = "tar.gz"
    else:
        ext = "tar.xz"
    filename = "node-v{}-{}-{}.{}".format(version, os_str, arch_str, ext)
    return "https://nodejs.org/dist/v{}/{}".format(version, filename)

# ---------------------------------------------------------------------------
# install_layout — strip top-level "node-v{version}-{os}-{arch}/" dir
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    platform = _node_platform(ctx)
    if not platform:
        return {"__type": "archive", "strip_prefix": "", "executable_paths": ["node"]}
    os_str, arch_str = platform[0], platform[1]
    strip = "node-v{}-{}-{}".format(version, os_str, arch_str)
    if ctx.platform.os == "windows":
        exe_paths = ["node.exe", "npm.cmd", "npx.cmd"]
    else:
        exe_paths = ["bin/node", "bin/npm", "bin/npx"]
    return {
        "__type":           "archive",
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
# Path queries + environment (using stdlib helpers)
# ---------------------------------------------------------------------------

paths = path_fns("node")
store_root       = paths["store_root"]
get_execute_path = bin_subdir_execute_path("node")
environment      = bin_subdir_env()

def deps(_ctx, _version):
    return []
