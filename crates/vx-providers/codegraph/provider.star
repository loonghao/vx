# provider.star - codegraph provider
#
# CodeGraph: Pre-indexed code knowledge graph for AI coding agents.
# Integrates with Claude Code, Cursor, Codex, Gemini CLI, etc.
# Releases: https://github.com/colbymchenry/codegraph/releases
#
# Asset format: codegraph-{os}-{arch}.{ext}
#   os:   win32, darwin, linux
#   arch: x64, arm64
#   ext:  zip (windows), tar.gz (darwin/linux)
# Tag format:  v{version}
#
# Bundled Node.js archive: top-level codegraph-{os}-{arch}/ directory
# contains node (runtime) and bin/codegraph (entry script).

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "post_extract_permissions",
     "path_fns",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "codegraph"
description = "CodeGraph - Pre-indexed code knowledge graph for AI coding agents"
homepage    = "https://github.com/colbymchenry/codegraph"
repository  = "https://github.com/colbymchenry/codegraph"
license     = "MIT"
ecosystem   = "ai"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("codegraph",
        version_pattern = "\\d+\\.\\d+\\.\\d+",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Platform mapping
#
# codegraph uses simple platform names (not Rust triples):
#   windows → win32
#   macos   → darwin
#   linux   → linux
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("win32", "x64"),
    "windows/arm64": ("win32", "arm64"),
    "macos/x64":     ("darwin", "x64"),
    "macos/arm64":   ("darwin", "arm64"),
    "linux/x64":     ("linux", "x64"),
    "linux/arm64":   ("linux", "arm64"),
}

def _codegraph_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# fetch_versions — from colbymchenry/codegraph releases
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix(
    "colbymchenry", "codegraph",
    tag_prefix = "v",
)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _codegraph_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://github.com/colbymchenry/codegraph/releases/download/v{}/codegraph-{}-{}.{}".format(
        version, os_str, arch_str, ext)

# ---------------------------------------------------------------------------
# install_layout
#
# Archive layout: codegraph-{os}-{arch}/
#   bin/codegraph      (Linux/macOS — shell script invoking bundled node)
#   bin/codegraph.cmd  (Windows — cmd wrapper)
#   node / node.exe    (bundled Node.js runtime)
#   lib/               (application code)
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    platform = _codegraph_platform(ctx)
    if not platform:
        return {
            "type":         "archive",
            "strip_prefix": "",
            "executable_paths": ["bin/codegraph"],
        }
    os_str, arch_str = platform
    strip = "codegraph-{}-{}".format(os_str, arch_str)
    if ctx.platform.os == "windows":
        exe_paths = ["bin/codegraph.cmd"]
    else:
        exe_paths = ["bin/codegraph"]
    return {
        "type":             "archive",
        "strip_prefix":     strip,
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# post_extract — ensure +x on Unix entry scripts
# ---------------------------------------------------------------------------

post_extract = post_extract_permissions([
    "bin/codegraph",
    "node",
])

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

paths = path_fns("codegraph")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
