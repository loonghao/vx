# provider.star - Rust provider
#
# Rust is managed via rustup. rustup-init is a single binary installer.
# After running rustup-init, rustc/cargo/rustfmt/clippy are available.
#
# Uses stdlib templates from @vx//stdlib:provider.star
#
# RFC 0040: Implements version_info() to map user-facing rustc versions
# to store layout and installer parameters. This enables O(1) version
# detection in `vx check` and clean `vx lock` behavior.

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:install.star", "set_permissions", "run_command")
load("@vx//stdlib:env.star",    "env_set", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "rust"
description = "Rust - A language empowering everyone to build reliable and efficient software"
homepage    = "https://www.rust-lang.org"
repository  = "https://github.com/rust-lang/rust"
license     = "MIT OR Apache-2.0"
ecosystem   = "rust"

# Supported package prefixes for ecosystem:package syntax (RFC 0027)
# Enables `vx cargo:<package>` for Rust crate installation via `cargo install`
package_prefixes = ["cargo"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    # Primary runtime: "rust" (executable: rustup). Store dir = "rust" via store_root().
    # bundled_with must use "rust" (not "rustup") so store_name() matches store_root().
    runtime_def("rust",
        executable      = "rustup",
        aliases         = ["rustup"],
        version_pattern = "rustup",
    ),
    bundled_runtime_def("rustc",   bundled_with = "rust",
        version_pattern = "rustc"),
    bundled_runtime_def("cargo",   bundled_with = "rust",
        version_pattern = "cargo"),
    bundled_runtime_def("rustfmt", bundled_with = "rust"),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["static.rust-lang.org"])

# ---------------------------------------------------------------------------
# fetch_versions — rustup GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("rust-lang", "rustup")

# ---------------------------------------------------------------------------
# Platform helpers
# rustup-init asset: rustup-init-{version}-{triple}[.exe]
# ---------------------------------------------------------------------------

_RUSTUP_TRIPLES = {
    "windows/x64":  "x86_64-pc-windows-msvc",
    "windows/x86":  "i686-pc-windows-msvc",
    "macos/x64":    "x86_64-apple-darwin",
    "macos/arm64":  "aarch64-apple-darwin",
    "linux/x64":    "x86_64-unknown-linux-musl",
    "linux/arm64":  "aarch64-unknown-linux-musl",
}

def _rustup_triple(ctx):
    return _RUSTUP_TRIPLES.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# RFC 0040: version_info — map rustc version to store layout
#
# Users write:   rust = "1.93.1"  (rustc version) in vx.toml
# We download:   latest rustup-init (installer, version irrelevant)
# We store to:   ~/.vx/store/rust/1.93.1/  (by rustc version!)
# We install:    --default-toolchain 1.93.1
#
# This enables O(1) check (directory existence) and eliminates the need
# for the passthrough/store-scan workaround in check.rs and lock.rs.
# ---------------------------------------------------------------------------

def version_info(_ctx, user_version):
    """Map user-facing rustc version to store layout and install parameters.

    Key insight: ANY rustup version can install ANY rustc version.
    So we always download the latest rustup installer and use it to install
    the specific rustc toolchain the user requested.

    Args:
        ctx: Provider context
        user_version: Version from vx.toml (e.g., "1.93.1", "stable", "1.85")

    Returns:
        dict with store_as, download_version, install_params
    """
    return {
        "store_as": user_version,      # ~/.vx/store/rust/1.93.1/
        "download_version": None,       # None = use latest rustup from fetch_versions()
        "install_params": {
            "toolchain": user_version,  # passed to post_extract as ctx.install_params
        },
    }

# ---------------------------------------------------------------------------
# download_url — rustup-init binary
# ---------------------------------------------------------------------------

def download_url(ctx, _version):
    # rustup-init is downloaded from the Rust CDN, not from versioned GitHub assets.
    # The CDN always serves the latest stable rustup-init for each platform triple;
    # the toolchain version is installed by running rustup-init with --default-toolchain.
    triple = _rustup_triple(ctx)
    if not triple:
        return None
    if ctx.platform.os == "windows":
        return "https://static.rust-lang.org/rustup/dist/{}/rustup-init.exe".format(triple)
    else:
        return "https://static.rust-lang.org/rustup/dist/{}/rustup-init".format(triple)

# ---------------------------------------------------------------------------
# install_layout — single binary installer
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    # The CDN asset is always named "rustup-init" (or "rustup-init.exe" on Windows).
    triple = _rustup_triple(ctx)
    if not triple:
        return None
    if ctx.platform.os == "windows":
        source = "rustup-init.exe"
        target = "rustup-init.exe"
    else:
        source = "rustup-init"
        target = "rustup-init"
    return {
        "type":               "binary",
        "source_name":        source,
        "target_name":        target,
        "target_dir":         "bin",
        "target_permissions": "755",
    }

# ---------------------------------------------------------------------------
# post_extract — set permissions + run rustup-init to install toolchain
#
# RFC 0040: Uses ctx.install_params["toolchain"] (set by version_info)
# instead of hardcoded "stable", so the correct rustc version is installed.
# ---------------------------------------------------------------------------

def post_extract(ctx, _version, install_dir):
    actions = []
    init_bin = "rustup-init.exe" if ctx.platform.os == "windows" else "rustup-init"
    if ctx.platform.os != "windows":
        actions.append(set_permissions("bin/" + init_bin, "755"))

    # Use toolchain from install_params (provided by version_info)
    # Falls back to "stable" for backward compatibility
    toolchain = "stable"
    if hasattr(ctx, "install_params") and ctx.install_params != None:
        t = ctx.install_params.get("toolchain")
        if t != None and t != "":
            toolchain = t

    actions.append(run_command(
        install_dir + "/bin/" + init_bin,

        args = ["-y", "--no-modify-path", "--default-toolchain", toolchain],
        env  = {
            "RUSTUP_HOME": install_dir + "/rustup",
            "CARGO_HOME":  install_dir + "/cargo",
        },
        on_failure = "error",
    ))
    return actions

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/rust"

def get_execute_path(ctx, _version):
    # ctx.runtime_name is the requested runtime (e.g. "cargo", "rustc", "rustfmt", "rust").
    # ctx.install_dir already includes the <platform> sub-directory
    # (e.g. ~/.vx/store/rust/1.29.0/windows-x64) where cargo/bin/ lives.
    runtime = ctx.runtime_name or "rust"
    exe_suffix = ".exe" if ctx.platform.os == "windows" else ""

    base = ctx.install_dir

    if runtime in ("rustc", "cargo", "rustfmt"):
        exe = runtime + exe_suffix
        return base + "/cargo/bin/" + exe
    # "rust" runtime → the rustup manager binary
    exe = "rustup" + exe_suffix
    return base + "/cargo/bin/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    # ctx.install_dir already includes the <platform> sub-directory
    # where rustup-init placed the cargo/rustup directories.
    base = ctx.install_dir
    return [
        env_set("RUSTUP_HOME", base + "/rustup"),
        env_set("CARGO_HOME",  base + "/cargo"),
        env_prepend("PATH",    base + "/cargo/bin"),
    ]

def deps(_ctx, _version):
    return []
