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
load("@vx//stdlib:install.star", "set_permissions", "run_command")

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

# ---------------------------------------------------------------------------
# post_extract — make rustup-init executable, then run it to install toolchain
#
# rustup-init is a self-contained installer binary. After download we need to:
#   1. Set executable permissions (Unix only)
#   2. Run it with -y --no-modify-path to install the stable toolchain
#
# The Rust runtime's run_command descriptor handles the actual execution.
# RUSTUP_HOME and CARGO_HOME are set to subdirs of install_dir so the
# toolchain is fully self-contained under vx's managed directory.
# ---------------------------------------------------------------------------

def post_extract(ctx, version, install_dir):
    """Set permissions and run rustup-init to install the Rust toolchain.

    Args:
        ctx:         Provider context
        version:     Installed rustup version string
        install_dir: Path to the installation directory

    Returns:
        List of post-extract actions
    """
    os = ctx["platform"]["os"]
    actions = []

    if os != "windows":
        # Make rustup-init executable on Unix
        actions.append(set_permissions("rustup-init", "755"))

    # Determine the init binary name
    init_bin = "rustup-init.exe" if os == "windows" else "rustup-init"

    # Run rustup-init to install the stable toolchain
    # -y                  : non-interactive
    # --no-modify-path    : don't touch shell profiles
    # --default-toolchain : install stable
    actions.append(run_command(
        install_dir + "/" + init_bin,
        args = [
            "-y",
            "--no-modify-path",
            "--default-toolchain", "stable",
        ],
        env = {
            "RUSTUP_HOME": install_dir + "/rustup",
            "CARGO_HOME":  install_dir + "/cargo",
        },
        on_failure = "error",
    ))

    return actions
