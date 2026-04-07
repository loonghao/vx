# provider.star - tokei provider
#
# tokei: Count your code, quickly.
# Releases: https://github.com/XAMPPRocky/tokei/releases
# Asset format: tokei-{triple}.tar.gz (Unix) / tokei-{triple}.exe (Windows)
# Tag format:   v{version}
#
# NOTE: tokei asset names embed the Rust target triple WITHOUT a version number.
# Windows ships a direct .exe binary; Unix ships a .tar.gz containing the binary.
#
# macOS arm64 has no native build; the x86_64 binary runs via Rosetta 2.
#
# Platform mapping:
#   windows/x64  → x86_64-pc-windows-msvc  (.exe — binary_install)
#   windows/x86  → i686-pc-windows-msvc    (.exe — binary_install)
#   macos/x64    → x86_64-apple-darwin     (.tar.gz — archive)
#   macos/arm64  → x86_64-apple-darwin     (.tar.gz — Rosetta 2 fallback)
#   linux/x64    → x86_64-unknown-linux-musl (.tar.gz — archive)
#   linux/arm64  → aarch64-unknown-linux-gnu (.tar.gz — archive)

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "path_fns")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "tokei"
description = "tokei - Count your code, quickly"
homepage    = "https://github.com/XAMPPRocky/tokei"
repository  = "https://github.com/XAMPPRocky/tokei"
license     = "MIT OR Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("tokei",
        version_pattern = "tokei \\d+",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "tokei \\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — standard GitHub releases with v-prefix tags
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("XAMPPRocky", "tokei")

# ---------------------------------------------------------------------------
# Platform → (triple, ext) mapping
#
# tokei asset names: tokei-{triple}.{ext}  (no version number in filename)
# macOS arm64 has no native build; falls back to x86_64 via Rosetta 2.
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":  ("x86_64-pc-windows-msvc",    "exe"),
    "windows/x86":  ("i686-pc-windows-msvc",      "exe"),
    "macos/x64":    ("x86_64-apple-darwin",        "tar.gz"),
    "macos/arm64":  ("x86_64-apple-darwin",        "tar.gz"),  # Rosetta 2 fallback
    "linux/x64":    ("x86_64-unknown-linux-musl",  "tar.gz"),
    "linux/arm64":  ("aarch64-unknown-linux-gnu",  "tar.gz"),
}

def _platform(ctx):
    return _PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    p = _platform(ctx)
    if not p:
        return None
    triple, ext = p
    asset = "tokei-{}.{}".format(triple, ext)
    return github_asset_url("XAMPPRocky", "tokei", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
#
# Windows: binary_install — direct .exe download, placed into bin/
# Unix:    archive        — .tar.gz, binary at archive root (no strip_prefix)
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    p = _platform(ctx)
    if not p:
        return None
    triple, _ext = p
    if ctx.platform.os == "windows":
        return {
            "__type":           "binary_install",
            "source_name":      "tokei-{}.exe".format(triple),
            "target_name":      "tokei.exe",
            "target_dir":       "bin",
            "executable_paths": ["bin/tokei.exe"],
        }
    return {
        "__type":           "archive",
        "strip_prefix":     "",
        "executable_paths": ["tokei"],
    }

# ---------------------------------------------------------------------------
# Path + environment helpers
# ---------------------------------------------------------------------------

_paths       = path_fns("tokei")
store_root   = _paths["store_root"]

def get_execute_path(ctx, _version):
    if ctx.platform.os == "windows":
        return ctx.install_dir + "/bin/tokei.exe"
    return ctx.install_dir + "/tokei"

def environment(ctx, _version):
    if ctx.platform.os == "windows":
        return [env_prepend("PATH", ctx.install_dir + "/bin")]
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
