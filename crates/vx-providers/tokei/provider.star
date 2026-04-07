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
# macOS arm64 (Apple Silicon) has no official binary build, and the x86_64 build
# does not run reliably via Rosetta 2 (TLS compatibility issue in old binaries).
# On macOS, Homebrew is provided as a fallback — it supplies a native arm64 build
# and can also be used on x64. The direct download is preferred on macOS x64.
#
# Platform mapping (direct download):
#   windows/x64  → x86_64-pc-windows-msvc  (.exe — binary_install)
#   windows/x86  → i686-pc-windows-msvc    (.exe — binary_install)
#   macos/x64    → x86_64-apple-darwin     (.tar.gz — archive)
#   linux/x64    → x86_64-unknown-linux-musl (.tar.gz — archive)
#   linux/arm64  → aarch64-unknown-linux-gnu (.tar.gz — archive)

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "path_fns",
     "system_install_strategies", "brew_install")
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
# macOS arm64 is intentionally absent: the x86_64 build fails via Rosetta 2
# due to TLS compatibility issues. macOS arm64 falls back to Homebrew below.
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":  ("x86_64-pc-windows-msvc",    "exe"),
    "windows/x86":  ("i686-pc-windows-msvc",      "exe"),
    "macos/x64":    ("x86_64-apple-darwin",        "tar.gz"),
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
# system_install — Homebrew fallback for macOS (including arm64)
# ---------------------------------------------------------------------------

system_install = system_install_strategies([
    brew_install("tokei", priority = 80),
])

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
