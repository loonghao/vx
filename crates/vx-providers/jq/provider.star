# provider.star - jq provider
#
# jq: Lightweight and flexible command-line JSON processor
# Tags use "jq-" prefix: "jq-1.8.1" → version "1.8.1"
# Asset: jq-{os}-{arch}[.exe]  (direct binary, no archive)
#
# Uses stdlib templates.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "fetch_versions_with_tag_prefix",
     "binary_layout", "path_fns")
load("@vx//stdlib:github.star", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "jq"
description = "Lightweight and flexible command-line JSON processor"
homepage    = "https://jqlang.github.io/jq/"
repository  = "https://github.com/jqlang/jq"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("jq",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "jq-\\d+"},
            {"command": "{executable} -n \"1+1\"", "name": "eval_check",
             "expected_output": "2"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — jq tags use "jq-" prefix
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix("jqlang", "jq", tag_prefix = "jq-")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_JQ_PLATFORMS = {
    "windows/x64":  ("windows", "amd64"),
    "windows/x86":  ("windows", "i386"),
    "macos/x64":    ("macos",   "amd64"),
    "macos/arm64":  ("macos",   "arm64"),
    "linux/x64":    ("linux",   "amd64"),
    "linux/arm64":  ("linux",   "arm64"),
}

def download_url(ctx, version):
    platform = _JQ_PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))
    if not platform:
        return None
    jq_os, jq_arch = platform[0], platform[1]
    if ctx.platform.os == "windows":
        asset = "jq-{}-{}.exe".format(jq_os, jq_arch)
    else:
        asset = "jq-{}-{}".format(jq_os, jq_arch)
    return github_asset_url("vx-org", "mirrors", "jq-" + version, asset)

# ---------------------------------------------------------------------------
# Layout + path functions (from stdlib)
# jq places binary in bin/ subdir
# ---------------------------------------------------------------------------

install_layout = binary_layout("jq")
_paths         = path_fns("jq")
store_root     = _paths["store_root"]

def get_execute_path(ctx, _version):
    exe = "jq.exe" if ctx.platform.os == "windows" else "jq"
    return ctx.install_dir + "/bin/" + exe

def environment(ctx, _version):
    return [{"op": "prepend", "key": "PATH", "value": ctx.install_dir + "/bin"}]
