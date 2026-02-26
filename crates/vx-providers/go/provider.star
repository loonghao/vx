# provider.star - Go programming language provider
#
# Version source: https://go.dev/dl/?mode=json (official API, no rate limiting)
# Bundled runtimes: gofmt (included in every Go release)
# Archive layout: go/ top-level dir, then bin/go and bin/gofmt
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def",
     "fetch_versions_from_api",
     "system_permissions",
     "bin_subdir_execute_path",
     "dep_def", "pre_run_ensure_deps")
load("@vx//stdlib:env.star", "env_set", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "go"
description = "Go - An open source programming language that makes it easy to build simple, reliable, and efficient software"
homepage    = "https://go.dev"
repository  = "https://github.com/golang/go"
license     = "BSD-3-Clause"
ecosystem   = "go"
aliases     = ["golang"]

# Supported package prefixes for ecosystem:package syntax (RFC 0027)
# Enables `vx go:<package>` for Go package installation via `go install`
package_prefixes = ["go"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("go",
        aliases = ["golang"],
        test_commands = [
            {"command": "{executable} version", "name": "version_check",
             "expected_output": "go\\d+\\.\\d+"},
            {"command": "{executable} env GOVERSION", "name": "env_check"},
        ],
    ),
    bundled_runtime_def("gofmt", bundled_with = "go",
        test_commands = [
            {"command": "{executable} -l .", "name": "list_check", "expect_success": True},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = system_permissions(extra_hosts = ["go.dev"])

# ---------------------------------------------------------------------------
# fetch_versions — go.dev official API (no rate limiting)
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_api(
    "https://go.dev/dl/?mode=json&include=all",
    "go_versions",
)

# ---------------------------------------------------------------------------
# Platform helpers
# Go uses: windows/darwin/linux × amd64/arm64/386/armv6l
# ---------------------------------------------------------------------------

_GO_PLATFORMS = {
    "windows/x64":  ("windows", "amd64"),
    "windows/x86":  ("windows", "386"),
    "macos/x64":    ("darwin",  "amd64"),
    "macos/arm64":  ("darwin",  "arm64"),
    "linux/x64":    ("linux",   "amd64"),
    "linux/arm64":  ("linux",   "arm64"),
    "linux/armv7":  ("linux",   "armv6l"),
}

def _go_platform(ctx):
    return _GO_PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url — go.dev
# Windows: go{version}.{os}-{arch}.zip
# Unix:    go{version}.{os}-{arch}.tar.gz
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _go_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform[0], platform[1]
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    filename = "go{}.{}-{}.{}".format(version, os_str, arch_str, ext)
    return "https://go.dev/dl/{}".format(filename)

# ---------------------------------------------------------------------------
# install_layout — strip top-level "go/" dir; executables in bin/
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    if ctx.platform.os == "windows":
        exe_paths = ["bin/go.exe", "bin/gofmt.exe"]
    else:
        exe_paths = ["bin/go", "bin/gofmt"]
    return {
        "type":             "archive",
        "strip_prefix":     "go",
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# environment — GOROOT + PATH prepend bin/
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    return [
        env_set("GOROOT", ctx.install_dir),
        env_prepend("PATH", ctx.install_dir + "/bin"),
    ]

# ---------------------------------------------------------------------------
# pre_run — ensure go mod download before `go run`
# ---------------------------------------------------------------------------

pre_run = pre_run_ensure_deps("go",
    trigger_args = ["run"],
    check_file   = "go.mod",
    lock_file    = "go.sum",
    install_dir  = "vendor",
)

# ---------------------------------------------------------------------------
# Path queries
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/go"

get_execute_path = bin_subdir_execute_path("go")

def post_install(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# deps — git recommended for module fetching
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return [
        dep_def("git", optional = True,
                reason = "Git is required for fetching Go modules"),
    ]
