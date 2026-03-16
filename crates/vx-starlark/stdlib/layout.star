# @vx//stdlib:layout.star
# Install layout, hook, path/env, and fetch_versions helpers for vx provider scripts
#
# This module provides helpers for:
#   - Declaring install layouts (archive, binary, bin-subdir)
#   - Post-extract and pre-run hooks
#   - PATH / environment helpers
#   - Standalone path function builders
#   - Non-GitHub fetch_versions helpers
#
# ┌─────────────────────────────────────────────────────────────────────────┐
# │  Layout builders                                                        │
# │  archive_layout()         Standalone archive install_layout builder     │
# │  binary_layout()          Standalone binary install_layout builder      │
# │  bin_subdir_layout()      Layout for tools with bin/ subdir (node/go)   │
# ├─────────────────────────────────────────────────────────────────────────┤
# │  Hook builders                                                          │
# │  post_extract_flatten()   Flatten versioned top-level dir after extract │
# │  post_extract_shim()      Create a shim executable after extract        │
# │  post_extract_permissions() Set Unix execute permissions after extract  │
# │  post_extract_combine()   Combine multiple post_extract hooks           │
# │  pre_run_ensure_deps()    Ensure project deps before running a command  │
# ├─────────────────────────────────────────────────────────────────────────┤
# │  Path / env helpers                                                     │
# │  bin_subdir_env()         PATH env for tools with bin/ subdir           │
# │  bin_subdir_execute_path() get_execute_path for bin/ subdir tools       │
# │  path_fns()               store_root + get_execute_path helpers         │
# │  path_env_fns()           environment + post_install helpers            │
# ├─────────────────────────────────────────────────────────────────────────┤
# │  fetch_versions helpers                                                 │
# │  fetch_versions_from_api()        Non-GitHub JSON API (nodejs.org, …)  │
# │  fetch_versions_with_tag_prefix() Non-standard tag prefix (bun-v…)     │
# └─────────────────────────────────────────────────────────────────────────┘

load("@vx//stdlib:install.star",  "flatten_dir", "create_shim", "set_permissions",
                                   "ensure_dependencies")
load("@vx//stdlib:http.star",     "fetch_json_versions", "github_releases")
load("@vx//stdlib:env.star",      "env_prepend")
load("@vx//stdlib:platform.star", "rust_triple", "go_os_arch", "archive_ext",
                                   "exe_suffix", "expand_asset")

# ---------------------------------------------------------------------------
# Internal aliases (keep private names for backward compat within this file)
# ---------------------------------------------------------------------------

def _rust_triple(ctx, linux_libc):
    return rust_triple(ctx, linux_libc)

def _go_os_arch(ctx):
    return go_os_arch(ctx)

def _archive_ext(ctx):
    return archive_ext(ctx)

def _exe_suffix(ctx):
    return exe_suffix(ctx)

def _expand_asset(template, ctx, version, triple = None, go_os = None, go_arch = None):
    return expand_asset(template, ctx, version, triple = triple,
                        go_os = go_os, go_arch = go_arch)

# ---------------------------------------------------------------------------
# Layout builders
# ---------------------------------------------------------------------------

def archive_layout(executable, strip_prefix = None):
    """Return an install_layout(ctx, version) function for archive installs.

    Use this when you write a custom `download_url` but still want the
    standard archive extraction behaviour.

    Args:
        executable:   Base executable name (without .exe; added automatically).
        strip_prefix: Top-level directory template to strip from the archive.
                      Placeholders: {version}, {vversion}, {triple}, {os}, {arch}, {ext}
                      None = no stripping (executable is at archive root).

    Returns:
        A function: install_layout(ctx, version) -> dict

    Example:
        # Archive root contains the binary directly
        install_layout = archive_layout("mytool")

        # Archive has a top-level dir "mytool-v1.0.0-x86_64-unknown-linux-musl/"
        install_layout = archive_layout(
            "mytool",
            strip_prefix = "mytool-{vversion}-{triple}",
        )
    """
    def install_layout(ctx, version):
        exe   = executable + _exe_suffix(ctx)
        strip = ""
        if strip_prefix != None:
            triple         = _rust_triple(ctx, "musl")
            go_os, go_arch = _go_os_arch(ctx)
            strip = _expand_asset(strip_prefix, ctx, version,
                                  triple  = triple  if triple  else "",
                                  go_os   = go_os   if go_os   else "",
                                  go_arch = go_arch if go_arch else "")
        return {
            "type":             "archive",
            "strip_prefix":     strip,
            "executable_paths": [exe, executable],
        }
    return install_layout

def binary_layout(executable):
    """Return an install_layout(ctx, version) function for single-binary installs.

    Use this when the download is a plain executable file (no archive).

    Args:
        executable: Base executable name (without .exe; added automatically).

    Returns:
        A function: install_layout(ctx, version) -> dict

    Example:
        install_layout = binary_layout("kubectl")
    """
    def install_layout(ctx, _version):
        exe = executable + _exe_suffix(ctx)
        return {
            "type":             "binary",
            "target_name":      exe,
            "target_dir":       "bin",
            "executable_paths": ["bin/" + exe, exe, executable],
        }
    return install_layout


def bin_subdir_layout(executables, strip_prefix = None):
    """Return an install_layout(ctx, version) function for tools with a bin/ subdir.

    Many language runtimes (Node.js, Go, Java) place executables in a `bin/`
    subdirectory on Unix but at the root on Windows. This helper handles that
    pattern automatically.

    Args:
        executables:  List of executable base names (without .exe or bin/ prefix).
                      e.g. ["node", "npm", "npx"]
        strip_prefix: Archive top-level directory template to strip.
                      Same placeholders as `_expand_asset`. None = no stripping.

    Returns:
        A function: install_layout(ctx, version) -> dict

    Example:
        # Node.js: bin/node on Unix, node.exe at root on Windows
        install_layout = bin_subdir_layout(
            ["node", "npm", "npx"],
            strip_prefix = "node-v{version}-{os_str}-{arch_str}",
        )

        # Go: bin/go, bin/gofmt on all platforms
        install_layout = bin_subdir_layout(["go", "gofmt"])
    """
    def _layout(ctx, version):
        os = ctx.platform.os
        if os == "windows":
            exe_paths = [e + ".exe" for e in executables] + executables
        else:
            exe_paths = ["bin/" + e for e in executables] + executables

        strip = ""
        if strip_prefix != None:
            go_os, go_arch = _go_os_arch(ctx)
            triple         = _rust_triple(ctx, "musl")
            strip = _expand_asset(strip_prefix, ctx, version,
                                  triple  = triple  if triple  else "",
                                  go_os   = go_os   if go_os   else "",
                                  go_arch = go_arch if go_arch else "")
        return {
            "type":             "archive",
            "strip_prefix":     strip,
            "executable_paths": exe_paths,
        }
    return _layout

# ---------------------------------------------------------------------------
# Post-extract hook builders
# ---------------------------------------------------------------------------

def post_extract_flatten(pattern = None):
    """Return a post_extract(ctx, version, install_dir) function that flattens
    a nested top-level directory after archive extraction.

    Use this when the archive extracts to a single versioned subdirectory
    (e.g. jdk-21.0.1+12/, ffmpeg-7.1-amd64-static/) and you want the
    contents moved directly into the install root.

    Args:
        pattern: Optional glob pattern to match the subdirectory name
                 (e.g. "jdk-*", "ffmpeg-*"). If None, flattens the single
                 subdirectory if exactly one exists.

    Returns:
        A function: post_extract(ctx, version, install_dir) -> list

    Example:
        # Java: flatten jdk-21.0.1+12/ -> install_dir/bin/java
        post_extract = post_extract_flatten(pattern="jdk-*")

        # FFmpeg Linux: flatten ffmpeg-7.1-amd64-static/ -> install_dir/ffmpeg
        post_extract = post_extract_flatten()
    """
    def _post_extract(_ctx, _version, _install_dir):
        return [flatten_dir(pattern = pattern)]
    return _post_extract

def post_extract_shim(shim_name, target_executable, args = None):
    """Return a post_extract(ctx, version, install_dir) function that creates
    a shim script after archive extraction.

    Use this when the archive does not include a standalone wrapper executable
    (e.g. bun ships only `bun`, not `bunx`).

    Args:
        shim_name:         Name of the shim to create (e.g. "bunx")
        target_executable: Executable the shim wraps (e.g. "bun")
        args:              Arguments to prepend (e.g. ["x"] for `bun x`)

    Returns:
        A function: post_extract(ctx, version, install_dir) -> list

    Example:
        # bun: create bunx -> bun x shim
        post_extract = post_extract_shim("bunx", "bun", args=["x"])
    """
    def _post_extract(_ctx, _version, _install_dir):
        return [create_shim(shim_name, target_executable, args = args)]
    return _post_extract

def post_extract_permissions(paths, mode = "755", unix_only = True):
    """Return a post_extract(ctx, version, install_dir) function that sets
    Unix file permissions after archive extraction.

    Use this when the archive may not preserve execute permissions on Unix
    (common in Docker/CI environments).

    Args:
        paths:     List of relative paths to chmod (e.g. ["bin/node", "bin/npm"])
        mode:      Unix permission mode string (default: "755")
        unix_only: If True (default), skip on Windows

    Returns:
        A function: post_extract(ctx, version, install_dir) -> list

    Example:
        # Node.js: ensure bundled tools have execute permissions
        post_extract = post_extract_permissions(
            ["bin/node", "bin/npm", "bin/npx", "bin/corepack"]
        )
    """
    def _post_extract(ctx, _version, _install_dir):
        if unix_only and ctx.platform.os == "windows":
            return []
        return [set_permissions(p, mode) for p in paths]
    return _post_extract

def post_extract_combine(hooks):
    """Combine multiple post_extract hook functions into one.

    Use this when you need both flattening AND shim creation, or any other
    combination of post_extract actions.

    Args:
        hooks: List of post_extract functions to combine (each returns a list)

    Returns:
        A function: post_extract(ctx, version, install_dir) -> list

    Example:
        post_extract = post_extract_combine([
            post_extract_flatten(pattern="jdk-*"),
            post_extract_permissions(["bin/java", "bin/javac"]),
        ])
    """
    def _post_extract(ctx, version, install_dir):
        result = []
        for hook in hooks:
            result = result + hook(ctx, version, install_dir)
        return result
    return _post_extract

# ---------------------------------------------------------------------------
# Pre-run hook builders
# ---------------------------------------------------------------------------

def pre_run_ensure_deps(package_manager, trigger_args = None,
                        check_file = "package.json",
                        lock_file = None,
                        install_dir = "node_modules"):
    """Return a pre_run(ctx, args, executable) function that ensures project
    dependencies are installed before running a command.

    Use this for package managers that need `install` to be run before
    `run` commands (npm, bun, uv, go mod download, etc.).

    Args:
        package_manager: Package manager executable (e.g. "npm", "bun", "uv")
        trigger_args:    List of first-arg values that trigger the check.
                         e.g. ["run", "run-script"] for npm.
                         If None, always triggers.
        check_file:      File that must exist for the check to apply
                         (e.g. "package.json", "pyproject.toml", "go.mod")
        lock_file:       Optional lock file to check for staleness
        install_dir:     Directory whose existence means deps are installed
                         (e.g. "node_modules", ".venv", "vendor")

    Returns:
        A function: pre_run(ctx, args, executable) -> list

    Example:
        # npm: ensure node_modules before `npm run`
        pre_run = pre_run_ensure_deps("npm",
            trigger_args = ["run", "run-script"],
            check_file   = "package.json",
            install_dir  = "node_modules",
        )

        # uv: ensure .venv before `uv run`
        pre_run = pre_run_ensure_deps("uv",
            trigger_args = ["run"],
            check_file   = "pyproject.toml",
            install_dir  = ".venv",
        )

        # go: ensure vendor before `go run`
        pre_run = pre_run_ensure_deps("go",
            trigger_args = ["run"],
            check_file   = "go.mod",
            lock_file    = "go.sum",
            install_dir  = "vendor",
        )
    """
    def _pre_run(_ctx, args, _executable):
        if trigger_args != None:
            if len(args) == 0 or args[0] not in trigger_args:
                return []
        return [ensure_dependencies(
            package_manager,
            check_file  = check_file,
            lock_file   = lock_file,
            install_dir = install_dir,
        )]
    return _pre_run

# ---------------------------------------------------------------------------
# fetch_versions helpers for non-GitHub APIs
# ---------------------------------------------------------------------------

def fetch_versions_from_api(url, transform):
    """Return a fetch_versions(ctx) function that fetches from a JSON API.

    Use this for tools with official version APIs (Node.js, Go, Java, etc.)
    that don't use GitHub releases. Avoids GitHub API rate limiting.

    Supported transform strategies (handled by the Rust runtime):
        "nodejs_org"         - https://nodejs.org/dist/index.json
        "go_versions"        - https://go.dev/dl/?mode=json
        "adoptium"           - https://api.adoptium.net/v3/info/available_releases
        "pypi"               - https://pypi.org/pypi/{package}/json
        "npm_registry"       - https://registry.npmjs.org/{package}
        "hashicorp_releases" - HashiCorp releases API

    Args:
        url:       The API URL to fetch
        transform: Named transform strategy (see above)

    Returns:
        A function: fetch_versions(ctx) -> descriptor

    Example:
        # Node.js official API
        fetch_versions = fetch_versions_from_api(
            "https://nodejs.org/dist/index.json",
            "nodejs_org",
        )

        # Go official API
        fetch_versions = fetch_versions_from_api(
            "https://go.dev/dl/?mode=json&include=all",
            "go_versions",
        )

        # Java Adoptium API
        fetch_versions = fetch_versions_from_api(
            "https://api.adoptium.net/v3/info/available_releases",
            "adoptium",
        )
    """
    def _fetch_versions(ctx):
        return fetch_json_versions(ctx, url, transform)
    return _fetch_versions

def fetch_versions_with_tag_prefix(owner, repo, tag_prefix,
                                   prereleases = False):
    """Return a fetch_versions(ctx) function for repos with non-standard tag prefixes.

    Use this when the GitHub release tags don't follow the standard "v{version}"
    pattern (e.g. bun uses "bun-v{version}", some tools use "{name}-{version}").

    Args:
        owner:      GitHub owner
        repo:       GitHub repo name
        tag_prefix: The prefix to strip from tag names to get the version.
                    e.g. "bun-v" strips "bun-v1.2.3" -> "1.2.3"
        prereleases: Include pre-release versions (default: False)

    Returns:
        A function: fetch_versions(ctx) -> descriptor

    Example:
        # bun: tags are "bun-v1.2.3"
        fetch_versions = fetch_versions_with_tag_prefix(
            "oven-sh", "bun", tag_prefix="bun-v"
        )
    """
    def _fetch_versions(ctx):
        releases = github_releases(ctx, owner, repo,
                                   include_prereleases = prereleases)
        return {
            "__type":           "github_versions",
            "source":           releases,
            "tag_key":          "tag_name",
            "strip_v_prefix":   False,
            "tag_prefix":       tag_prefix,
            "skip_prereleases": not prereleases,
        }
    return _fetch_versions

# ---------------------------------------------------------------------------
# environment helpers for tools with bin/ subdirectory
# ---------------------------------------------------------------------------

def bin_subdir_env(extra_env = None):
    """Return an environment(ctx, version) function that prepends bin/ to PATH.

    Use this for tools that place executables in a `bin/` subdirectory on Unix
    but at the root on Windows (Node.js, Go, Java pattern).

    Args:
        extra_env: Additional env ops (list of dicts from env.star)

    Returns:
        A function: environment(ctx, version) -> list

    Example:
        environment = bin_subdir_env()
        environment = bin_subdir_env(extra_env=[env_set("GOROOT", ctx.install_dir)])
    """
    def _environment(ctx, _version):
        os = ctx.platform.os
        if os == "windows":
            bin_dir = ctx.install_dir
        else:
            bin_dir = ctx.install_dir + "/bin"
        ops = [env_prepend("PATH", bin_dir)]
        if extra_env != None:
            ops = ops + extra_env
        return ops
    return _environment

def bin_subdir_execute_path(executable):
    """Return a get_execute_path(ctx, version) function for bin/ subdir tools.

    Args:
        executable: Base executable name (without .exe)

    Returns:
        A function: get_execute_path(ctx, version) -> str

    Example:
        get_execute_path = bin_subdir_execute_path("node")
        get_execute_path = bin_subdir_execute_path("go")
    """
    def _get_execute_path(ctx, _version):
        os = ctx.platform.os
        if os == "windows":
            return ctx.install_dir + "/" + executable + ".exe"
        else:
            return ctx.install_dir + "/bin/" + executable
    return _get_execute_path

# ---------------------------------------------------------------------------
# Standalone path/env builders
# ---------------------------------------------------------------------------

def path_fns(store_name, executable = None):
    """Return store_root and get_execute_path functions for a provider.

    Use this when you need the RFC 0037 path functions but are writing
    other functions (download_url, install_layout) manually.

    Args:
        store_name:  Store directory name
        executable:  Executable name; defaults to `store_name`

    Returns:
        A dict with keys: store_root, get_execute_path

    Example:
        _paths = path_fns("ripgrep", executable="rg")
        store_root       = _paths["store_root"]
        get_execute_path = _paths["get_execute_path"]
    """
    exe_name = executable if executable != None else store_name

    def _store_root(ctx):
        return ctx.vx_home + "/store/" + store_name

    def _get_execute_path(ctx, _version):
        return ctx.install_dir + "/" + exe_name + _exe_suffix(ctx)

    return {
        "store_root":       _store_root,
        "get_execute_path": _get_execute_path,
    }

def path_env_fns(extra_env = None):
    """Return environment and post_install functions that prepend PATH.

    Args:
        extra_env: Additional env ops beyond PATH prepend.

    Returns:
        A dict with keys: environment, post_install

    Example:
        _env = path_env_fns()
        environment  = _env["environment"]
        post_install = _env["post_install"]
    """
    def _environment(ctx, _version):
        ops = [env_prepend("PATH", ctx.install_dir)]
        if extra_env != None:
            ops = ops + extra_env
        return ops

    def _post_install(_ctx, _version):
        return None

    return {
        "environment":  _environment,
        "post_install": _post_install,
    }
