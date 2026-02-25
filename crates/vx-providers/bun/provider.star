# provider.star - bun provider
#
# Bun: Incredibly fast JavaScript runtime, bundler, test runner, and package manager
# Tags use "bun-v{version}" prefix; asset: bun-{os}-{arch}.zip (no version in name)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "github_permissions",
     "fetch_versions_with_tag_prefix",
     "post_extract_shim", "pre_run_ensure_deps")
load("@vx//stdlib:github.star", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "bun"
description = "Incredibly fast JavaScript runtime, bundler, test runner, and package manager"
homepage    = "https://bun.sh"
repository  = "https://github.com/oven-sh/bun"
license     = "MIT"
ecosystem   = "nodejs"

# Supported package prefixes for ecosystem:package syntax (RFC 0027)
# Enables `vx bun:<package>` and `vx bunx:<package>` for Bun package execution
package_prefixes = ["bun", "bunx"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("bun",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "^\\d+\\.\\d+\\.\\d+"},
            {"command": "{executable} -e \"console.log('ok')\"", "name": "eval_check",
             "expected_output": "ok"},
        ],
    ),
    bundled_runtime_def("bunx", bundled_with = "bun",
        executable     = "bun",
        command_prefix = ["x"],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — bun uses "bun-v{version}" tag format
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix("oven-sh", "bun", tag_prefix = "bun-v")

# ---------------------------------------------------------------------------
# Platform helpers
# bun asset: bun-{os}-{arch}.zip  (windows/darwin/linux × x64/aarch64)
# ---------------------------------------------------------------------------

_BUN_PLATFORMS = {
    "windows/x64":  ("windows", "x64"),
    "macos/x64":    ("darwin",  "x64"),
    "macos/arm64":  ("darwin",  "aarch64"),
    "linux/x64":    ("linux",   "x64"),
    "linux/arm64":  ("linux",   "aarch64"),
}

def _bun_platform(ctx):
    return _BUN_PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url — bun-{os}-{arch}.zip, tag = "bun-v{version}"
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _bun_platform(ctx)
    if not platform:
        return None
    bun_os, bun_arch = platform[0], platform[1]
    asset = "bun-{}-{}.zip".format(bun_os, bun_arch)
    return github_asset_url("oven-sh", "bun", "bun-v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — strip top-level "bun-{os}-{arch}/" dir
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    platform = _bun_platform(ctx)
    exe      = "bun.exe" if ctx.platform.os == "windows" else "bun"
    strip    = "bun-{}-{}".format(platform[0], platform[1]) if platform else ""
    return {
        "type":             "archive",
        "strip_prefix":     strip,
        "executable_paths": [exe, "bun"],
    }

# ---------------------------------------------------------------------------
# post_extract — create bunx shim (bun ships only `bun`, not `bunx`)
# ---------------------------------------------------------------------------

post_extract = post_extract_shim("bunx", "bun", args = ["x"])

# ---------------------------------------------------------------------------
# pre_run — ensure node_modules before `bun run`
# ---------------------------------------------------------------------------

pre_run = pre_run_ensure_deps("bun",
    trigger_args = ["run"],
    check_file   = "package.json",
    lock_file    = "bun.lockb",
    install_dir  = "node_modules",
)

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/bun"

def get_execute_path(ctx, _version):
    exe = "bun.exe" if ctx.platform.os == "windows" else "bun"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
