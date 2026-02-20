# provider.star - fd (fd-find) provider
#
# Inheritance pattern (Level 2):
#   - fetch_versions: fully inherited from github.star
#   - download_url:   overridden — asset uses "fd-v{version}-{triple}.{ext}" naming
#
# Rust: sharkdp/fd config.rs -> ~30 lines Starlark

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "fd"

def description():
    return "fd - A simple, fast and user-friendly alternative to 'find'"

def homepage():
    return "https://github.com/sharkdp/fd"

def repository():
    return "https://github.com/sharkdp/fd"

def license():
    return "MIT OR Apache-2.0"

def ecosystem():
    return "devtools"

def aliases():
    return ["fd-find"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "fd",
        "executable":  "fd",
        "description": "A simple, fast and user-friendly alternative to 'find'",
        "aliases":     ["fd-find"],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — fully inherited
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("sharkdp", "fd")

# ---------------------------------------------------------------------------
# download_url — override
#
# Asset naming: "fd-v{version}-{triple}.{ext}"
# Tag:          "v{version}"
# Linux uses musl for portability.
# ---------------------------------------------------------------------------

def _fd_triple(ctx):
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]
    triples = {
        "windows/x64":   "x86_64-pc-windows-msvc",
        "windows/arm64": "aarch64-pc-windows-msvc",
        "macos/x64":     "x86_64-apple-darwin",
        "macos/arm64":   "aarch64-apple-darwin",
        "linux/x64":     "x86_64-unknown-linux-musl",
        "linux/arm64":   "aarch64-unknown-linux-gnu",
    }
    return triples.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    triple = _fd_triple(ctx)
    if not triple:
        return None
    os  = ctx["platform"]["os"]
    ext = "zip" if os == "windows" else "tar.gz"
    # Asset: "fd-v0.10.2-x86_64-unknown-linux-musl.tar.gz"
    asset = "fd-v{}-{}.{}".format(version, triple, ext)
    return github_asset_url("sharkdp", "fd", "v{}".format(version), asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os  = ctx["platform"]["os"]
    triple = _fd_triple(ctx)
    exe = "fd.exe" if os == "windows" else "fd"
    # fd archives contain a subdirectory: "fd-v{version}-{triple}/"
    strip = "fd-v{}-{}".format(version, triple) if triple else ""
    return {
        "type":             "archive",
        "strip_prefix":     strip,
        "executable_paths": [exe, "fd"],
    }

def environment(ctx, version, install_dir):
    return {"PATH": install_dir}


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    return "{vx_home}/store/fd"

def get_execute_path(ctx, version):
    os = ctx["platform"]["os"]
    if os == "windows":
        return "{install_dir}/fd.exe"
    else:
        return "{install_dir}/fd"

def post_install(ctx, version, install_dir):
    return None
