# provider.star - Rust provider
#
# Rust is managed via rustup, the official Rust toolchain installer.
# rustup itself is distributed as GitHub releases.
# rustc, cargo, rustfmt, clippy are all provided by rustup after installation.
#
# Version source: https://github.com/rust-lang/rustup/releases
#
# Inheritance pattern: Level 2 (custom download_url for rustup binary)

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "rust"

def description():
    return "Rust - A language empowering everyone to build reliable and efficient software"

def homepage():
    return "https://www.rust-lang.org"

def repository():
    return "https://github.com/rust-lang/rust"

def license():
    return "MIT OR Apache-2.0"

def ecosystem():
    return "rust"

def aliases():
    return []

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "rustup",
        "executable":  "rustup",
        "description": "The Rust toolchain installer",
        "priority":    100,
    },
    {
        "name":        "rustc",
        "executable":  "rustc",
        "description": "Rust compiler (provided by rustup)",
        "aliases":     ["rust"],
        "bundled_with": "rustup",
    },
    {
        "name":        "cargo",
        "executable":  "cargo",
        "description": "Rust package manager and build tool (provided by rustup)",
        "bundled_with": "rustup",
    },
    {
        "name":        "rustfmt",
        "executable":  "rustfmt",
        "description": "Rust code formatter (provided by rustup)",
        "bundled_with": "rustup",
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com", "static.rust-lang.org"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — rustup GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("rust-lang", "rustup")

# ---------------------------------------------------------------------------
# download_url — rustup binary (platform-specific)
# ---------------------------------------------------------------------------

def _rustup_asset(ctx):
    """Map vx platform to rustup binary asset name."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    assets = {
        "windows/x64":   "rustup-init.exe",
        "windows/x86":   "rustup-init.exe",
        "macos/x64":     "rustup-init",
        "macos/arm64":   "rustup-init",
        "linux/x64":     "rustup-init",
        "linux/arm64":   "rustup-init",
    }
    return assets.get("{}/{}".format(os, arch))

def _rustup_triple(ctx):
    """Map vx platform to Rust target triple for rustup binary path."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    triples = {
        "windows/x64":   "x86_64-pc-windows-msvc",
        "windows/x86":   "i686-pc-windows-msvc",
        "macos/x64":     "x86_64-apple-darwin",
        "macos/arm64":   "aarch64-apple-darwin",
        "linux/x64":     "x86_64-unknown-linux-musl",
        "linux/arm64":   "aarch64-unknown-linux-musl",
    }
    return triples.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the rustup-init download URL.

    rustup releases are at:
      https://github.com/rust-lang/rustup/releases/download/1.27.1/
        rustup-init-1.27.1-x86_64-pc-windows-msvc.exe

    Args:
        ctx:     Provider context
        version: rustup version string, e.g. "1.27.1"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    triple = _rustup_triple(ctx)
    if not triple:
        return None

    os = ctx["platform"]["os"]
    if os == "windows":
        asset = "rustup-init-{}-{}.exe".format(version, triple)
    else:
        asset = "rustup-init-{}-{}".format(version, triple)

    return github_asset_url("rust-lang", "rustup", version, asset)

# ---------------------------------------------------------------------------
# install_layout — rustup-init is a single binary installer
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx["platform"]["os"]
    exe = "rustup-init.exe" if os == "windows" else "rustup-init"
    return {
        "type":             "binary",
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "RUSTUP_HOME": install_dir + "/rustup",
        "CARGO_HOME":  install_dir + "/cargo",
        "PATH":        install_dir + "/cargo/bin",
    }

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    """rustup has no external dependencies."""
    return []
