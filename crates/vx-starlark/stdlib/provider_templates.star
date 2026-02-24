# @vx//stdlib:provider_templates.star
# High-level provider templates for vx provider scripts
#
# This module provides ready-made templates that eliminate boilerplate in
# provider.star files. Choose the template that matches your tool's release
# pattern, then only override what differs.
#
# ┌─────────────────────────────────────────────────────────────────────────┐
# │  github_rust_provider()   GitHub releases, Rust target triple naming    │
# │  github_go_provider()     GitHub releases, Go-style os/arch naming      │
# │  github_binary_provider() GitHub releases, single binary (no archive)   │
# │  system_provider()        System package manager (winget/brew/apt)      │
# └─────────────────────────────────────────────────────────────────────────┘
#
# Quick-start (Rust triple, the most common pattern):
#
#   load("@vx//stdlib:provider_templates.star",
#        "github_rust_provider")
#
#   _p               = github_rust_provider("owner", "mytool",
#                          asset = "mytool-{vversion}-{triple}.{ext}")
#   fetch_versions   = _p["fetch_versions"]
#   download_url     = _p["download_url"]
#   install_layout   = _p["install_layout"]
#   store_root       = _p["store_root"]
#   get_execute_path = _p["get_execute_path"]
#   post_install     = _p["post_install"]
#   environment      = _p["environment"]
#   deps             = _p["deps"]

load("@vx//stdlib:github.star",   "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",      "env_prepend")
load("@vx//stdlib:platform.star", "os_to_go", "arch_to_go")

# ---------------------------------------------------------------------------
# Internal: platform helpers
# ---------------------------------------------------------------------------

# Rust target triples — musl on Linux (portable, no glibc version dependency)
_RUST_TRIPLES_MUSL = {
    "windows/x64":   "x86_64-pc-windows-msvc",
    "windows/arm64": "aarch64-pc-windows-msvc",
    "macos/x64":     "x86_64-apple-darwin",
    "macos/arm64":   "aarch64-apple-darwin",
    "linux/x64":     "x86_64-unknown-linux-musl",
    "linux/arm64":   "aarch64-unknown-linux-musl",
}

# Rust target triples — gnu on Linux
_RUST_TRIPLES_GNU = {
    "windows/x64":   "x86_64-pc-windows-msvc",
    "windows/arm64": "aarch64-pc-windows-msvc",
    "macos/x64":     "x86_64-apple-darwin",
    "macos/arm64":   "aarch64-apple-darwin",
    "linux/x64":     "x86_64-unknown-linux-gnu",
    "linux/arm64":   "aarch64-unknown-linux-gnu",
}

def _rust_triple(ctx, linux_libc):
    """Resolve the Rust target triple for the current platform."""
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    if linux_libc == "gnu":
        return _RUST_TRIPLES_GNU.get(key)
    return _RUST_TRIPLES_MUSL.get(key)

def _go_os_arch(ctx):
    """Resolve Go-style (os, arch) for the current platform."""
    return (os_to_go(ctx.platform.os), arch_to_go(ctx.platform.arch))

def _archive_ext(ctx):
    """'zip' on Windows, 'tar.gz' elsewhere."""
    return "zip" if ctx.platform.os == "windows" else "tar.gz"

def _exe_suffix(ctx):
    """'.exe' on Windows, '' elsewhere."""
    return ".exe" if ctx.platform.os == "windows" else ""

def _expand_asset(template, ctx, version, triple = None, go_os = None, go_arch = None):
    """Expand an asset filename template with platform-specific values.

    Placeholders:
        {version}  - version without 'v' prefix  (e.g. "1.0.0")
        {vversion} - version with 'v' prefix      (e.g. "v1.0.0")
        {triple}   - Rust target triple           (e.g. "x86_64-unknown-linux-musl")
        {os}       - Go GOOS                      (e.g. "linux", "darwin", "windows")
        {arch}     - Go GOARCH                    (e.g. "amd64", "arm64")
        {ext}      - archive extension            (e.g. "zip" or "tar.gz")
        {exe}      - executable suffix            (e.g. ".exe" or "")
    """
    ext = _archive_ext(ctx)
    exe = _exe_suffix(ctx)

    s = template
    s = s.replace("{version}",  version)
    s = s.replace("{vversion}", "v" + version)
    s = s.replace("{ext}",      ext)
    s = s.replace("{exe}",      exe)
    if triple != None:
        s = s.replace("{triple}", triple)
    if go_os != None:
        s = s.replace("{os}",   go_os)
    if go_arch != None:
        s = s.replace("{arch}", go_arch)
    return s

# ---------------------------------------------------------------------------
# Internal: standard provider function set
# ---------------------------------------------------------------------------

def _std_provider_fns(store_name, exe_name, path_env, extra_env):
    """Build the standard set of provider functions (store_root, get_execute_path,
    post_install, environment, install_layout, deps) that are identical across
    most providers.
    """
    def _store_root(ctx):
        return ctx.vx_home + "/store/" + store_name

    def _get_execute_path(ctx, _version):
        return ctx.install_dir + "/" + exe_name + _exe_suffix(ctx)

    def _post_install(_ctx, _version):
        return None

    def _environment(ctx, _version):
        ops = [env_prepend("PATH", ctx.install_dir)] if path_env else []
        if extra_env != None:
            ops = ops + extra_env
        return ops

    def _install_layout(ctx, _version):
        exe = exe_name + _exe_suffix(ctx)
        return {
            "type":             "archive",
            "strip_prefix":     "",
            "executable_paths": [exe, exe_name],
        }

    def _deps(_ctx, _version):
        return []

    return {
        "store_root":       _store_root,
        "get_execute_path": _get_execute_path,
        "post_install":     _post_install,
        "environment":      _environment,
        "install_layout":   _install_layout,
        "deps":             _deps,
    }

# ---------------------------------------------------------------------------
# Template 1: github_rust_provider
#   GitHub releases with Rust target triple asset naming
#   e.g. "mytool-v1.0.0-x86_64-unknown-linux-musl.tar.gz"
# ---------------------------------------------------------------------------

def github_rust_provider(owner, repo, asset,
                         executable = None,
                         store = None,
                         tag_prefix = "v",
                         linux_libc = "musl",
                         prereleases = False,
                         strip_prefix = None,
                         path_env = True,
                         extra_env = None):
    """Provider template for GitHub releases using Rust target triple naming.

    This is the most common pattern for Rust-written CLI tools.

    Asset template placeholders:
        {version}  - version without 'v' prefix  (e.g. "1.0.0")
        {vversion} - version with 'v' prefix      (e.g. "v1.0.0")
        {triple}   - Rust target triple           (e.g. "x86_64-unknown-linux-musl")
        {ext}      - archive extension            (e.g. "zip" or "tar.gz")
        {exe}      - executable suffix            (e.g. ".exe" or "")

    Args:
        owner:       GitHub owner (e.g. "BurntSushi")
        repo:        GitHub repo name (e.g. "ripgrep")
        asset:       Asset filename template (e.g. "rg-{version}-{triple}.{ext}")
        executable:  Executable name; defaults to `repo`
        store:       Store directory name; defaults to `repo`
        tag_prefix:  Tag prefix before version (default: "v", use "" for no prefix)
        linux_libc:  Linux C library: "musl" (default, portable) or "gnu"
        prereleases: Include pre-release versions (default: False)
        strip_prefix: Archive top-level directory template to strip.
                      Same placeholders as `asset`. None = no stripping.
        path_env:    Prepend install_dir to PATH (default: True)
        extra_env:   Additional env ops (list of dicts from env.star)

    Returns:
        A dict with all standard provider functions:
        fetch_versions, download_url, install_layout,
        store_root, get_execute_path, post_install, environment, deps

    Example:
        # ripgrep: "rg-14.1.1-x86_64-unknown-linux-musl.tar.gz"
        _p = github_rust_provider(
            "BurntSushi", "ripgrep",
            asset      = "rg-{version}-{triple}.{ext}",
            executable = "rg",
        )

        # jj: "jj-v0.38.0-x86_64-pc-windows-msvc.zip"
        _p = github_rust_provider(
            "jj-vcs", "jj",
            asset = "jj-{vversion}-{triple}.{ext}",
        )

        # just: no 'v' in tag, "just-1.40.0-x86_64-unknown-linux-musl.tar.gz"
        _p = github_rust_provider(
            "casey", "just",
            asset      = "just-{version}-{triple}.{ext}",
            tag_prefix = "",
        )
    """
    exe_name   = executable if executable != None else repo
    store_name = store      if store      != None else repo

    def _fetch_versions(ctx):
        from_github = make_fetch_versions(owner, repo, prereleases)
        return from_github(ctx)

    def _download_url(ctx, version):
        triple = _rust_triple(ctx, linux_libc)
        if not triple:
            return None
        tag   = tag_prefix + version
        fname = _expand_asset(asset, ctx, version, triple = triple)
        return github_asset_url(owner, repo, tag, fname)

    def _install_layout(ctx, version):
        exe   = exe_name + _exe_suffix(ctx)
        strip = ""
        if strip_prefix != None:
            triple = _rust_triple(ctx, linux_libc)
            strip  = _expand_asset(strip_prefix, ctx, version,
                                   triple = triple if triple else "")
        return {
            "type":             "archive",
            "strip_prefix":     strip,
            "executable_paths": [exe, exe_name],
        }

    fns = _std_provider_fns(store_name, exe_name, path_env, extra_env)
    fns["fetch_versions"] = _fetch_versions
    fns["download_url"]   = _download_url
    fns["install_layout"] = _install_layout
    return fns

# ---------------------------------------------------------------------------
# Template 2: github_go_provider
#   GitHub releases with Go-style os/arch naming
#   e.g. "mytool_1.0.0_linux_amd64.tar.gz"
# ---------------------------------------------------------------------------

def github_go_provider(owner, repo, asset,
                       executable = None,
                       store = None,
                       tag_prefix = "v",
                       prereleases = False,
                       strip_prefix = None,
                       path_env = True,
                       extra_env = None):
    """Provider template for GitHub releases using Go-style os/arch naming.

    Use this for tools written in Go that follow the standard goreleaser
    naming convention (darwin/linux/windows + amd64/arm64).

    Asset template placeholders:
        {version}  - version without 'v' prefix  (e.g. "1.0.0")
        {vversion} - version with 'v' prefix      (e.g. "v1.0.0")
        {os}       - Go GOOS                      (e.g. "linux", "darwin", "windows")
        {arch}     - Go GOARCH                    (e.g. "amd64", "arm64")
        {ext}      - archive extension            (e.g. "zip" or "tar.gz")
        {exe}      - executable suffix            (e.g. ".exe" or "")

    Args:
        owner:       GitHub owner (e.g. "dagu-org")
        repo:        GitHub repo name (e.g. "dagu")
        asset:       Asset filename template (e.g. "dagu_{version}_{os}_{arch}.{ext}")
        executable:  Executable name; defaults to `repo`
        store:       Store directory name; defaults to `repo`
        tag_prefix:  Tag prefix before version (default: "v", use "" for no prefix)
        prereleases: Include pre-release versions (default: False)
        strip_prefix: Archive top-level directory template to strip.
                      Same placeholders as `asset`. None = no stripping.
        path_env:    Prepend install_dir to PATH (default: True)
        extra_env:   Additional env ops (list of dicts from env.star)

    Returns:
        A dict with all standard provider functions.

    Example:
        # dagu: "dagu_1.14.5_linux_amd64.tar.gz"
        _p = github_go_provider(
            "dagu-org", "dagu",
            asset = "dagu_{version}_{os}_{arch}.tar.gz",
        )

        # gh CLI: "gh_2.67.0_linux_amd64.tar.gz" with top-level dir
        _p = github_go_provider(
            "cli", "cli",
            asset        = "gh_{version}_{os}_{arch}.{ext}",
            executable   = "gh",
            strip_prefix = "gh_{version}_{os}_{arch}",
        )
    """
    exe_name   = executable if executable != None else repo
    store_name = store      if store      != None else repo

    def _fetch_versions(ctx):
        from_github = make_fetch_versions(owner, repo, prereleases)
        return from_github(ctx)

    def _download_url(ctx, version):
        go_os, go_arch = _go_os_arch(ctx)
        if not go_os:
            return None
        tag   = tag_prefix + version
        fname = _expand_asset(asset, ctx, version,
                              go_os = go_os, go_arch = go_arch)
        return github_asset_url(owner, repo, tag, fname)

    def _install_layout(ctx, version):
        exe   = exe_name + _exe_suffix(ctx)
        strip = ""
        if strip_prefix != None:
            go_os, go_arch = _go_os_arch(ctx)
            strip = _expand_asset(strip_prefix, ctx, version,
                                  go_os  = go_os   if go_os   else "",
                                  go_arch = go_arch if go_arch else "")
        return {
            "type":             "archive",
            "strip_prefix":     strip,
            "executable_paths": [exe, exe_name],
        }

    fns = _std_provider_fns(store_name, exe_name, path_env, extra_env)
    fns["fetch_versions"] = _fetch_versions
    fns["download_url"]   = _download_url
    fns["install_layout"] = _install_layout
    return fns

# ---------------------------------------------------------------------------
# Template 3: github_binary_provider
#   GitHub releases, single binary download (no archive extraction)
#   e.g. kubectl, helm (some platforms)
# ---------------------------------------------------------------------------

def github_binary_provider(owner, repo, asset,
                            executable = None,
                            store = None,
                            tag_prefix = "v",
                            prereleases = False,
                            path_env = True,
                            extra_env = None):
    """Provider template for GitHub releases that ship a single binary file.

    Use this for tools distributed as a plain executable (no archive).
    The downloaded file is placed directly in the install directory.

    Asset template placeholders:
        {version}  - version without 'v' prefix  (e.g. "1.0.0")
        {vversion} - version with 'v' prefix      (e.g. "v1.0.0")
        {triple}   - Rust target triple           (e.g. "x86_64-unknown-linux-musl")
        {os}       - Go GOOS                      (e.g. "linux", "darwin", "windows")
        {arch}     - Go GOARCH                    (e.g. "amd64", "arm64")
        {exe}      - executable suffix            (e.g. ".exe" or "")

    Args:
        owner:       GitHub owner
        repo:        GitHub repo name
        asset:       Asset filename template (e.g. "kubectl{exe}")
        executable:  Executable name; defaults to `repo`
        store:       Store directory name; defaults to `repo`
        tag_prefix:  Tag prefix before version (default: "v", use "" for no prefix)
        prereleases: Include pre-release versions (default: False)
        path_env:    Prepend install_dir to PATH (default: True)
        extra_env:   Additional env ops (list of dicts from env.star)

    Returns:
        A dict with all standard provider functions.

    Example:
        # kubectl: single binary per platform
        _p = github_binary_provider(
            "kubernetes", "kubectl",
            asset = "kubectl_{version}_{os}_{arch}{exe}",
        )
    """
    exe_name   = executable if executable != None else repo
    store_name = store      if store      != None else repo

    def _fetch_versions(ctx):
        from_github = make_fetch_versions(owner, repo, prereleases)
        return from_github(ctx)

    def _download_url(ctx, version):
        go_os, go_arch = _go_os_arch(ctx)
        triple = _rust_triple(ctx, "musl")
        tag    = tag_prefix + version
        fname  = _expand_asset(asset, ctx, version,
                               triple  = triple  if triple  else "",
                               go_os   = go_os   if go_os   else "",
                               go_arch = go_arch if go_arch else "")
        return github_asset_url(owner, repo, tag, fname)

    def _install_layout(ctx, _version):
        exe = exe_name + _exe_suffix(ctx)
        return {
            "type":             "binary",
            "executable_paths": [exe, exe_name],
        }

    fns = _std_provider_fns(store_name, exe_name, path_env, extra_env)
    fns["fetch_versions"] = _fetch_versions
    fns["download_url"]   = _download_url
    fns["install_layout"] = _install_layout
    return fns

# ---------------------------------------------------------------------------
# Template 4: system_provider
#   Tools installed via system package managers (winget/brew/apt/choco)
#   No download URL — the Rust runtime delegates to the package manager.
# ---------------------------------------------------------------------------

def system_provider(store_name, executable = None,
                    path_env = True, extra_env = None):
    """Provider template for system-managed tools (winget, brew, apt, etc.).

    Use this for tools that are installed via the OS package manager rather
    than downloaded from GitHub. The `download_url` always returns None;
    the Rust runtime uses `prepare_execution` / `system_install` instead.

    Args:
        store_name:  Store directory name (e.g. "7zip")
        executable:  Executable name; defaults to `store_name`
        path_env:    Prepend install_dir to PATH (default: True)
        extra_env:   Additional env ops (list of dicts from env.star)

    Returns:
        A dict with all standard provider functions.
        `fetch_versions` and `download_url` return empty list / None respectively.

    Example:
        _p = system_provider("7zip", executable="7z")
        store_root       = _p["store_root"]
        get_execute_path = _p["get_execute_path"]
        environment      = _p["environment"]
    """
    exe_name = executable if executable != None else store_name

    def _fetch_versions(_ctx):
        return []

    def _download_url(_ctx, _version):
        return None

    def _install_layout(_ctx, _version):
        return None

    fns = _std_provider_fns(store_name, exe_name, path_env, extra_env)
    fns["fetch_versions"] = _fetch_versions
    fns["download_url"]   = _download_url
    fns["install_layout"] = _install_layout
    return fns
