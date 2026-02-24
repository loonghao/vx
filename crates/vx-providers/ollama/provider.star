# provider.star - Ollama provider
#
# Ollama uses simple platform names (not Rust triples):
#   macOS: "darwin" (universal), Linux: "linux-amd64/arm64", Windows: "windows-amd64/arm64"
# Archive: .tgz (Unix), .zip (Windows)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "post_extract_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "ollama"
description = "Ollama - Get up and running with large language models locally"
homepage    = "https://ollama.com"
repository  = "https://github.com/ollama/ollama"
license     = "MIT"
ecosystem   = "ai"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("ollama",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "ollama version"},
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

fetch_versions = make_fetch_versions("ollama", "ollama", include_prereleases = False)

# ---------------------------------------------------------------------------
# Platform helpers
# macOS: universal "darwin"; Linux/Windows: "{os}-{arch}"
# ---------------------------------------------------------------------------

_OLLAMA_TARGETS = {
    "windows/x64":  ("windows-amd64", "zip"),
    "windows/arm64":("windows-arm64", "zip"),
    "macos/x64":    ("darwin",        "tgz"),
    "macos/arm64":  ("darwin",        "tgz"),
    "linux/x64":    ("linux-amd64",   "tgz"),
    "linux/arm64":  ("linux-arm64",   "tgz"),
}

def _ollama_target(ctx):
    return _OLLAMA_TARGETS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    target = _ollama_target(ctx)
    if not target:
        return None
    target_str, ext = target[0], target[1]
    asset = "ollama-{}.{}".format(target_str, ext)
    return github_asset_url("ollama", "ollama", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "ollama.exe" if ctx.platform.os == "windows" else "ollama"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "ollama"],
    }

# ---------------------------------------------------------------------------
# post_extract — set +x on Unix
# ---------------------------------------------------------------------------

post_extract = post_extract_permissions(["ollama"])

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/ollama"

def get_execute_path(ctx, _version):
    exe = "ollama.exe" if ctx.platform.os == "windows" else "ollama"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
