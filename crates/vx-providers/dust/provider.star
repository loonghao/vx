# provider.star - dust provider
#
# dust: A more intuitive version of du written in Rust
# Releases: https://github.com/bootandy/dust/releases
# Asset format: dust-v{version}-{triple}.{ext}
# Tag format:   v{version}
#
# Note: No aarch64-apple-darwin (macOS ARM64) build is published.
#       macOS ARM64 falls back to x86_64-apple-darwin (runs via Rosetta 2).

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "fetch_versions_with_tag_prefix",
     "path_fns")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "dust"
description = "dust - A more intuitive version of du written in Rust"
homepage    = "https://github.com/bootandy/dust"
repository  = "https://github.com/bootandy/dust"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [runtime_def("dust", version_pattern="[Dd]ust \\d+")]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Platform mapping
# Note: No aarch64-apple-darwin build — macOS ARM uses x86_64 via Rosetta 2
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("x86_64-pc-windows-msvc",    "zip"),
    "windows/arm64": ("x86_64-pc-windows-msvc",    "zip"),
    "macos/x64":     ("x86_64-apple-darwin",       "tar.gz"),
    "macos/arm64":   ("x86_64-apple-darwin",       "tar.gz"),  # no arm64 build; Rosetta 2
    "linux/x64":     ("x86_64-unknown-linux-musl", "tar.gz"),
    "linux/arm64":   ("aarch64-unknown-linux-musl","tar.gz"),
}

# ---------------------------------------------------------------------------
# Provider functions
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix("bootandy", "dust", tag_prefix = "v")

def _platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

def download_url(ctx, version):
    p = _platform(ctx)
    if not p:
        return None
    triple, ext = p
    fname = "dust-v{}-{}.{}".format(version, triple, ext)
    return "https://github.com/bootandy/dust/releases/download/v{}/{}".format(version, fname)

def install_layout(ctx, version):
    p = _platform(ctx)
    if not p:
        return None
    triple, _ext = p
    exe = "dust.exe" if ctx.platform.os == "windows" else "dust"
    strip = "dust-v{}-{}".format(version, triple)
    return {
        "__type":           "archive",
        "strip_prefix":     strip,
        "executable_paths": [exe, "dust"],
    }

paths = path_fns("dust")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
