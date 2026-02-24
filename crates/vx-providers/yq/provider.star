# provider.star - yq provider
#
# yq: portable command-line YAML, JSON, XML, CSV, TOML processor
# Asset: yq_{os}_{arch}[.exe]  (direct binary, no archive)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "yq"
description = "yq - a portable command-line YAML, JSON, XML, CSV, TOML and properties processor"
homepage    = "https://github.com/mikefarah/yq"
repository  = "https://github.com/mikefarah/yq"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("yq",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "yq \\(https://github.com/mikefarah/yq\\) version"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("mikefarah", "yq")

# ---------------------------------------------------------------------------
# download_url — direct binary: yq_{os}_{arch}[.exe]
# ---------------------------------------------------------------------------

_YQ_PLATFORMS = {
    "windows/x64":  ("windows", "amd64"),
    "windows/arm64":("windows", "arm64"),
    "macos/x64":    ("darwin",  "amd64"),
    "macos/arm64":  ("darwin",  "arm64"),
    "linux/x64":    ("linux",   "amd64"),
    "linux/arm64":  ("linux",   "arm64"),
    "linux/arm":    ("linux",   "arm"),
    "linux/x86":    ("linux",   "386"),
}

def download_url(ctx, version):
    platform = _YQ_PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))
    if not platform:
        return None
    yq_os, yq_arch = platform[0], platform[1]
    ext   = ".exe" if ctx.platform.os == "windows" else ""
    asset = "yq_{}_{}{}".format(yq_os, yq_arch, ext)
    return github_asset_url("mikefarah", "yq", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — single binary
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "yq.exe" if ctx.platform.os == "windows" else "yq"
    return {
        "type":             "binary",
        "rename_to":        exe,
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/yq"

def get_execute_path(ctx, _version):
    exe = "yq.exe" if ctx.platform.os == "windows" else "yq"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
