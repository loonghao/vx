# provider.star - Go provider
#
# Go - The Go programming language
# Downloads from go.dev/dl
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "github_permissions", "dep_def",
     "bin_subdir_layout", "bin_subdir_execute_path",
     "fetch_versions_from_api", "path_fns")
load("@vx//stdlib:env.star",    "env_set", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "go"
description = "Go - The Go programming language"
homepage    = "https://go.dev"
repository  = "https://github.com/golang/go"
license     = "BSD-3-Clause"
ecosystem   = "go"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("go",
        aliases         = ["golang"],
        version_pattern = "go\\d+\\.\\d+",
        test_commands = [
            {"command": "{executable} version", "name": "version_check",
             "expected_output": "go version go"},
        ],
    ),
    bundled_runtime_def("gofmt", "go",
        description     = "Go source code formatter",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["go.dev", "dl.google.com"])

# ---------------------------------------------------------------------------
# fetch_versions — from go.dev official API
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_api(
    "https://go.dev/dl/?mode=json&include=all",
    "go_versions",
)

# ---------------------------------------------------------------------------
# Platform helpers
# Go uses: go{version}.{os}-{arch}.{ext}
# ---------------------------------------------------------------------------

_GO_PLATFORMS = {
    "windows/x64":   ("windows", "amd64"),
    "windows/arm64": ("windows", "arm64"),
    "macos/x64":     ("darwin",  "amd64"),
    "macos/arm64":   ("darwin",  "arm64"),
    "linux/x64":     ("linux",   "amd64"),
    "linux/arm64":   ("linux",   "arm64"),
}

def _go_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _GO_PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _go_platform(ctx)
    if not platform:
        return None
    go_os, go_arch = platform[0], platform[1]
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://go.dev/dl/go{}.{}-{}.{}".format(version, go_os, go_arch, ext)

# ---------------------------------------------------------------------------
# install_layout — Go uses bin/ subdir on all platforms
# Go archives have a top-level "go/" directory
# ---------------------------------------------------------------------------

install_layout = bin_subdir_layout(["go", "gofmt"], strip_prefix = "go")

# ---------------------------------------------------------------------------
# Path queries + environment (using stdlib helpers)
# ---------------------------------------------------------------------------

_paths = path_fns("go")
store_root = _paths["store_root"]
get_execute_path = bin_subdir_execute_path("go")

def environment(ctx, _version):
    os = ctx.platform.os
    bin_dir = ctx.install_dir if os == "windows" else ctx.install_dir + "/bin"
    return [
        env_prepend("PATH", bin_dir),
        env_set("GOROOT", ctx.install_dir),
    ]

# ---------------------------------------------------------------------------
# deps — git recommended for module fetching
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("git", optional = True,
                reason = "Git is required for fetching Go modules"),
    ]
