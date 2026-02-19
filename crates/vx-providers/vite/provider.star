# provider.star - Vite provider
#
# Vite is a frontend build tool distributed as an npm package.
# `vx vite` is routed to `vx npx vite` (npm package alias pattern).
#
# Inheritance pattern: Level 3 (npm package alias, no direct download)

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "vite"

def description():
    return "Vite - Next generation frontend build tool"

def homepage():
    return "https://vitejs.dev"

def repository():
    return "https://github.com/vitejs/vite"

def license():
    return "MIT"

def ecosystem():
    return "nodejs"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "vite",
        "executable":  "vite",
        "description": "Vite frontend build tool",
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["registry.npmjs.org"],
    "fs":   [],
    "exec": ["npx", "node"],
}

# ---------------------------------------------------------------------------
# Package alias configuration
# ---------------------------------------------------------------------------

package_alias = {
    "ecosystem": "npm",
    "package":   "vite",
}

# ---------------------------------------------------------------------------
# fetch_versions — npm registry
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch Vite versions from npm registry."""
    data = ctx["http"]["get_json"](
        "https://registry.npmjs.org/vite"
    )
    versions_map = data.get("versions", {})
    dist_tags = data.get("dist-tags", {})
    latest = dist_tags.get("latest", "")

    versions = []
    for v in versions_map.keys():
        prerelease = "-" in v  # alpha/beta/rc contain hyphen
        versions.append({
            "version":    v,
            "lts":        v == latest,
            "prerelease": prerelease,
        })
    return versions

# ---------------------------------------------------------------------------
# download_url — not applicable (npm package)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    return None

# ---------------------------------------------------------------------------
# deps — requires node (with version constraints per vite version)
# ---------------------------------------------------------------------------

def deps(ctx, version):
    """Vite requires Node.js with version-specific constraints."""
    parts = version.split(".")
    major = int(parts[0]) if parts else 0

    if major >= 5:
        node_version = ">=18"
    elif major >= 3:
        node_version = ">=14.18"
    else:
        node_version = ">=12"

    return [
        {"runtime": "node", "version": node_version,
         "reason": "Vite {} requires Node.js {}".format(version, node_version)},
    ]
