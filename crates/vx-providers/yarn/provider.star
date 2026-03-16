# provider.star - yarn provider
#
# Yarn: Fast, reliable, and secure dependency management
# Asset: yarn-v{version}.tar.gz (Unix archive)
# Requires Node.js (version-dependent)
# Windows prefers system installation.
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "pre_run_ensure_deps",
     "system_install_strategies", "winget_install", "choco_install")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "yarn"
description = "Fast, reliable, and secure dependency management"
homepage    = "https://yarnpkg.com"
repository  = "https://github.com/yarnpkg/yarn"
license     = "BSD-2-Clause"
ecosystem   = "nodejs"

package_prefixes = ["yarn"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("yarn",
        aliases  = ["yarnpkg"],
        priority = 80,
        system_paths = [
            "C:/Program Files (x86)/Yarn/bin/yarn.cmd",
            "C:/Program Files/Yarn/bin/yarn.cmd",
            "/usr/local/bin/yarn",
            "/usr/bin/yarn",
        ],
        version_pattern = "^\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(exec_cmds = ["winget", "choco"])

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("yarnpkg", "yarn")

# ---------------------------------------------------------------------------
# download_url — Unix archive only
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    if ctx.platform.os == "windows":
        return None
    asset = "yarn-v{}.tar.gz".format(version)
    return github_asset_url("yarnpkg", "yarn", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — strip top-level "yarn-v{version}/" dir
# ---------------------------------------------------------------------------

def install_layout(_ctx, version):
    return {
        "type":             "archive",
        "strip_prefix":     "yarn-v{}".format(version),
        "executable_paths": ["bin/yarn", "bin/yarn.js"],
    }

# ---------------------------------------------------------------------------
# pre_run — ensure node_modules before `yarn run`
# ---------------------------------------------------------------------------

pre_run = pre_run_ensure_deps("yarn",
    trigger_args = ["run"],
    check_file   = "package.json",
    install_dir  = "node_modules",
)

# ---------------------------------------------------------------------------
# system_install — Windows package managers
# ---------------------------------------------------------------------------

system_install = system_install_strategies([
    winget_install("Yarn.Yarn", priority = 90),
    choco_install("yarn", priority = 80),
])

# ---------------------------------------------------------------------------
# deps — version-based Node.js dependency
# ---------------------------------------------------------------------------

def deps(_ctx, version):
    parts = version.split(".")
    major = int(parts[0]) if parts else 1
    if major >= 4:
        return [{"runtime": "node", "version": ">=18"}]
    elif major >= 2:
        return [{"runtime": "node", "version": ">=16.10"}]
    else:
        return [{"runtime": "node", "version": ">=12"}]

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/yarn"


def get_execute_path(ctx, _version):
    exe = "yarn.cmd" if ctx.platform.os == "windows" else "yarn"
    return ctx.install_dir + "/bin/" + exe


def post_install(_ctx, _version):
    return None


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]
