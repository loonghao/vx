# provider.star - Go programming language provider
#
# Version source: https://go.dev/dl/?mode=json (official API, no rate limiting)
# Bundled runtimes: gofmt (included in every Go release)
#
# Inheritance pattern: Level 1 (fully custom - uses go.dev API, not GitHub)
#
# Go releases: https://go.dev/dl/
#
# Hooks:
#   pre_run: run `go mod download` before `go run` when go.mod exists

load("@vx//stdlib:install.star", "ensure_dependencies")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "go"

def description():
    return "Go - An open source programming language that makes it easy to build simple, reliable, and efficient software"

def homepage():
    return "https://go.dev"

def repository():
    return "https://github.com/golang/go"

def license():
    return "BSD-3-Clause"

def ecosystem():
    return "go"

def aliases():
    return ["golang"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "go",
        "executable":  "go",
        "description": "Go programming language runtime",
        "aliases":     ["golang"],
        "priority":    100,
    },
    {
        "name":        "gofmt",
        "executable":  "gofmt",
        "description": "Go source code formatter (bundled with Go)",
        "bundled_with": "go",
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["go.dev"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — uses go.dev official API
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch Go versions from the official go.dev API.

    Uses https://go.dev/dl/?mode=json which provides:
    - Stable and unstable releases
    - File metadata per platform
    - No rate limiting
    """
    releases = ctx["http"]["get_json"]("https://go.dev/dl/?mode=json&include=all")

    versions = []
    seen = {}
    for release in releases:
        v = release.get("version", "")
        # Strip "go" prefix: "go1.21.0" -> "1.21.0"
        if v.startswith("go"):
            v = v[2:]

        if not v or v in seen:
            continue
        seen[v] = True

        stable = release.get("stable", False)
        versions.append({
            "version":    v,
            "lts":        stable,
            "prerelease": not stable,
        })

    return versions

# ---------------------------------------------------------------------------
# download_url — go.dev official download
# ---------------------------------------------------------------------------

def _go_platform(ctx):
    """Map vx platform to Go platform/arch strings."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    platforms = {
        "windows/x64":   ("windows", "amd64"),
        "windows/x86":   ("windows", "386"),
        "macos/x64":     ("darwin",  "amd64"),
        "macos/arm64":   ("darwin",  "arm64"),
        "linux/x64":     ("linux",   "amd64"),
        "linux/arm64":   ("linux",   "arm64"),
        "linux/armv7":   ("linux",   "armv6l"),
    }
    key = "{}/{}".format(os, arch)
    return platforms.get(key)

def download_url(ctx, version):
    """Build the Go download URL from go.dev.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'go' prefix, e.g. "1.21.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _go_platform(ctx)
    if not platform:
        return None

    os_str, arch_str = platform[0], platform[1]
    os = ctx["platform"]["os"]

    if os == "windows":
        # Windows: zip archive
        # e.g. https://go.dev/dl/go1.21.0.windows-amd64.zip
        filename = "go{}.{}-{}.zip".format(version, os_str, arch_str)
    else:
        # Unix: tar.gz archive
        # e.g. https://go.dev/dl/go1.21.0.linux-amd64.tar.gz
        filename = "go{}.{}-{}.tar.gz".format(version, os_str, arch_str)

    return "https://go.dev/dl/{}".format(filename)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx["platform"]["os"]

    if os == "windows":
        exe_paths = ["bin/go.exe", "bin/gofmt.exe"]
    else:
        exe_paths = ["bin/go", "bin/gofmt"]

    return {
        "type":             "archive",
        "strip_prefix":     "go",   # Go archives contain a top-level "go/" directory
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "GOROOT": install_dir,
        "PATH":   install_dir + "/bin",
    }

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    """Go recommends git for module fetching."""
    return [
        {"runtime": "git", "version": "*", "optional": True,
         "reason": "Git is required for fetching Go modules"},
    ]

# ---------------------------------------------------------------------------
# pre_run — ensure go mod download before `go run`
# ---------------------------------------------------------------------------

def pre_run(ctx, args, executable):
    """Ensure Go module dependencies are downloaded before running.

    For `go run` commands, checks if go.mod exists and vendor/ does not,
    then runs `go mod download` to ensure all dependencies are available.

    Args:
        ctx:        Provider context
        args:       Command-line arguments passed to go
        executable: Path to the go executable

    Returns:
        List of pre-run actions
    """
    if len(args) > 0 and args[0] == "run":
        return [
            ensure_dependencies(
                "go",
                check_file  = "go.mod",
                lock_file   = "go.sum",
                install_dir = "vendor",
            ),
        ]
    return []


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    return "{vx_home}/store/go"

def get_execute_path(ctx, version):
    os = ctx["platform"]["os"]
    if os == "windows":
        return "{install_dir}/go.exe"
    else:
        return "{install_dir}/go"

def post_install(ctx, version, install_dir):
    return None
