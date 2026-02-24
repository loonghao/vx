# provider.star - Rust provider
#
# Rust is managed via rustup. rustup-init is a single binary installer.
# After running rustup-init, rustc/cargo/rustfmt/clippy are available.
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
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

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("rustup",
        version_pattern = "rustup",
    ),
    bundled_runtime_def("rustc",   bundled_with = "rustup",
        aliases         = ["rust"],
        version_pattern = "rustc"),
    bundled_runtime_def("cargo",   bundled_with = "rustup",
        version_pattern = "cargo"),
    bundled_runtime_def("rustfmt", bundled_with = "rustup"),
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
# download_url — rustup-init binary
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    triple = _rustup_triple(ctx)
    if not triple:
        return None
    if ctx.platform.os == "windows":
        asset = "rustup-init-{}-{}.exe".format(version, triple)
    else:
        asset = "rustup-init-{}-{}".format(version, triple)
    return github_asset_url("rust-lang", "rustup", version, asset)

# ---------------------------------------------------------------------------
# install_layout — single binary installer
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "rustup-init.exe" if ctx.platform.os == "windows" else "rustup-init"
    return {
        "type":             "binary",
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# post_extract — set permissions + run rustup-init to install toolchain
# ---------------------------------------------------------------------------

def post_extract(ctx, _version, install_dir):
    actions = []
    if ctx.platform.os != "windows":
        actions.append(set_permissions("rustup-init", "755"))
    init_bin = "rustup-init.exe" if ctx.platform.os == "windows" else "rustup-init"
    actions.append(run_command(
        install_dir + "/" + init_bin,
        args = ["-y", "--no-modify-path", "--default-toolchain", "stable"],
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
    exe = "rustup.exe" if ctx.platform.os == "windows" else "rustup"
    return ctx.install_dir + "/cargo/bin/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [
        env_set("RUSTUP_HOME", ctx.install_dir + "/rustup"),
        env_set("CARGO_HOME",  ctx.install_dir + "/cargo"),
        env_prepend("PATH",    ctx.install_dir + "/cargo/bin"),
    ]

def deps(_ctx, _version):
    return []
