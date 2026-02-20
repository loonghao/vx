# provider.star - yarn provider
#
# Yarn: Fast, reliable, and secure dependency management
# Inheritance pattern: Level 2 (custom download_url for yarn's archive naming)
#   - fetch_versions: inherited from github.star
#   - download_url:   custom (yarn-v{version}.tar.gz)
#   - deps:           version-based Node.js dependency
#
# yarn releases: https://github.com/yarnpkg/yarn/releases
# Asset format: yarn-v{version}.tar.gz

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:install.star", "ensure_dependencies")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "yarn"

def description():
    return "Fast, reliable, and secure dependency management"

def homepage():
    return "https://yarnpkg.com"

def repository():
    return "https://github.com/yarnpkg/yarn"

def license():
    return "BSD-2-Clause"

def ecosystem():
    return "nodejs"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "yarn",
        "executable":  "yarn",
        "description": "Yarn package manager",
        "aliases":     ["yarnpkg"],
        "priority":    80,
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
# fetch_versions — inherited
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("yarnpkg", "yarn")

# ---------------------------------------------------------------------------
# download_url — custom
#
# yarn asset naming: yarn-v{version}.tar.gz (cross-platform, JS-based)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build the yarn download URL.

    Yarn 1.x is a cross-platform JS package, same archive for all platforms.

    Args:
        ctx:     Provider context
        version: Version string, e.g. "1.22.22"

    Returns:
        Download URL string
    """
    asset = "yarn-v{}.tar.gz".format(version)
    tag = "v{}".format(version)
    return github_asset_url("yarnpkg", "yarn", tag, asset)

# ---------------------------------------------------------------------------
# deps — version-based Node.js dependency
# ---------------------------------------------------------------------------

def deps(ctx, version):
    """Declare Node.js dependency based on yarn version.

    Yarn 1.x requires Node.js 12+
    Yarn 2.x-3.x requires Node.js 16.10+
    Yarn 4.x requires Node.js 18+
    """
    parts = version.split(".")
    major = int(parts[0]) if parts else 1

    if major >= 4:
        return [{"runtime": "node", "version": ">=18"}]
    elif major >= 2:
        return [{"runtime": "node", "version": ">=16.10"}]
    else:
        return [{"runtime": "node", "version": ">=12"}]

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    return {
        "type":             "archive",
        "strip_prefix":     "yarn-v{}".format(version),
        "executable_paths": ["bin/yarn.js", "bin/yarn"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir + "/bin",
    }

# ---------------------------------------------------------------------------
# pre_run — ensure node_modules before `yarn run`
# ---------------------------------------------------------------------------

def pre_run(ctx, args, executable):
    """Ensure project dependencies are installed before running yarn scripts.

    For `yarn run` commands, checks if node_modules exists and runs
    `yarn install` if not.

    Args:
        ctx:        Provider context
        args:       Command-line arguments passed to yarn
        executable: Path to the yarn executable

    Returns:
        List of pre-run actions
    """
    if len(args) > 0 and args[0] == "run":
        return [
            ensure_dependencies(
                "yarn",
                check_file  = "package.json",
                install_dir = "node_modules",
            ),
        ]
    return []
