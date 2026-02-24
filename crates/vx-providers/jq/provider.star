# provider.star - jq provider
#
# jq: Lightweight and flexible command-line JSON processor
# Tags use "jq-" prefix: "jq-1.8.1" → version "1.8.1"
# Asset: jq-{os}-{arch}[.exe]  (direct binary, no archive)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:github.star", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

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
# download_url — direct binary: jq-{os}-{arch}[.exe]
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
    return github_asset_url("jqlang", "jq", "jq-" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — single binary
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "jq.exe" if ctx.platform.os == "windows" else "jq"
    return {
        "type":               "binary",
        "target_name":        exe,
        "target_dir":         "bin",
        "target_permissions": "755",
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/jq"

def get_execute_path(ctx, _version):
    exe = "jq.exe" if ctx.platform.os == "windows" else "jq"
    return ctx.install_dir + "/bin/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def deps(_ctx, _version):
    return []
