# @vx//stdlib:install.star
# Installation layout descriptors for vx provider scripts
#
# Design: Starlark scripts are pure computation — they do NOT perform real
# installation. Instead, functions like msi_install() return a descriptor dict
# that the Rust runtime interprets to perform the actual installation I/O.
#
# This keeps Starlark sandboxed and testable, while the Rust layer handles
# all real I/O (download, extraction, msiexec, filesystem, etc.).
#
# Usage:
#   load("@vx//stdlib:install.star", "msi_install", "archive_install", "binary_install")
#
# Example (MSI provider):
#   load("@vx//stdlib:install.star", "msi_install")
#   load("@vx//stdlib:platform.star", "is_windows")
#
#   def download_url(ctx, version):
#       if not is_windows(ctx):
#           return None
#       return "https://example.com/tool-{}.msi".format(version)
#
#   def install_layout(ctx, version):
#       if not is_windows(ctx):
#           return None
#       url = download_url(ctx, version)
#       return msi_install(url, executable_paths = ["bin/tool.exe"])

# ---------------------------------------------------------------------------
# MSI installer descriptor (Windows only)
# ---------------------------------------------------------------------------

def msi_install(url, executable_paths = None, strip_prefix = None, extra_args = None):
    """Return an MSI installation descriptor for the Rust runtime to execute.

    Uses msiexec /a (administrative install) to extract the MSI contents to
    the target directory without modifying the system registry.

    This function does NOT perform any real installation. It returns a descriptor
    dict that the Rust runtime interprets to run msiexec.

    Args:
        url:              Download URL for the .msi file
        executable_paths: List of relative paths to executables within the
                          extracted MSI (e.g. ["bin/tool.exe", "tool.exe"]).
                          If None, the Rust runtime will auto-detect executables.
        strip_prefix:     Directory prefix to strip from extracted paths
                          (e.g. "PFiles/MyTool" if msiexec extracts there).
                          If None, no stripping is performed.
        extra_args:       Extra msiexec command-line properties
                          (e.g. ["ADDLOCAL=ALL"]).

    Returns:
        An install descriptor dict consumed by the Rust runtime.

    Example:
        return msi_install(
            "https://example.com/tool-1.0.msi",
            executable_paths = ["bin/tool.exe"],
            strip_prefix = "PFiles/Tool",
        )
    """
    descriptor = {
        "__type":    "msi_install",
        "url":       url,
    }
    if executable_paths != None:
        descriptor["executable_paths"] = executable_paths
    if strip_prefix != None:
        descriptor["strip_prefix"] = strip_prefix
    if extra_args != None:
        descriptor["extra_args"] = extra_args
    return descriptor

# ---------------------------------------------------------------------------
# Archive installer descriptor (ZIP, TAR.GZ, TAR.XZ, etc.)
# ---------------------------------------------------------------------------

def archive_install(url, strip_prefix = None, executable_paths = None):
    """Return an archive installation descriptor for the Rust runtime to execute.

    Supports ZIP, TAR.GZ, TAR.XZ, TAR.BZ2 archives. The format is auto-detected
    from the URL file extension.

    This function does NOT perform any real installation. It returns a descriptor
    dict that the Rust runtime interprets to download and extract the archive.

    Args:
        url:              Download URL for the archive file
        strip_prefix:     Directory prefix to strip from extracted paths
                          (e.g. "tool-1.0.0-linux-x64" for archives that
                          contain a top-level directory).
                          If None, no stripping is performed.
        executable_paths: List of relative paths to executables within the
                          extracted archive (e.g. ["bin/tool", "tool"]).
                          If None, the Rust runtime will auto-detect executables.

    Returns:
        An install descriptor dict consumed by the Rust runtime.

    Example:
        return archive_install(
            "https://example.com/tool-1.0-linux-x64.tar.gz",
            strip_prefix = "tool-1.0-linux-x64",
            executable_paths = ["bin/tool"],
        )
    """
    descriptor = {
        "__type": "archive_install",
        "url":    url,
    }
    if strip_prefix != None:
        descriptor["strip_prefix"] = strip_prefix
    if executable_paths != None:
        descriptor["executable_paths"] = executable_paths
    return descriptor

# ---------------------------------------------------------------------------
# Binary installer descriptor (single executable file)
# ---------------------------------------------------------------------------

def binary_install(url, executable_name = None, permissions = "755"):
    """Return a binary installation descriptor for the Rust runtime to execute.

    Downloads a single executable file directly (no archive extraction needed).

    This function does NOT perform any real installation. It returns a descriptor
    dict that the Rust runtime interprets to download the binary.

    Args:
        url:             Download URL for the binary file
        executable_name: Target filename for the downloaded binary.
                         If None, the filename is derived from the URL.
        permissions:     Unix file permissions (default: "755").
                         Ignored on Windows.

    Returns:
        An install descriptor dict consumed by the Rust runtime.

    Example:
        return binary_install(
            "https://example.com/tool-linux-x64",
            executable_name = "tool",
        )
    """
    descriptor = {
        "__type":     "binary_install",
        "url":        url,
        "permissions": permissions,
    }
    if executable_name != None:
        descriptor["executable_name"] = executable_name
    return descriptor

# ---------------------------------------------------------------------------
# Convenience: platform-aware install layout
# ---------------------------------------------------------------------------

def platform_install(ctx, windows_url = None, macos_url = None, linux_url = None,
                     strip_prefix = None, executable_paths = None,
                     windows_msi = False):
    """Return the appropriate install descriptor for the current platform.

    Automatically selects the right URL and install method based on the
    current platform. MSI is used for Windows when windows_msi=True.

    Args:
        ctx:              Provider context dict (injected by vx runtime)
        windows_url:      Download URL for Windows
        macos_url:        Download URL for macOS
        linux_url:        Download URL for Linux
        strip_prefix:     Directory prefix to strip (for archive installs)
        executable_paths: List of relative paths to executables
        windows_msi:      If True and on Windows, use msi_install() instead
                          of archive_install() for the Windows URL.

    Returns:
        An install descriptor dict, or None if no URL for current platform.

    Example:
        def install_layout(ctx, version):
            return platform_install(
                ctx,
                windows_url = "https://example.com/tool-{}.msi".format(version),
                macos_url   = "https://example.com/tool-{}-macos.tar.gz".format(version),
                linux_url   = "https://example.com/tool-{}-linux.tar.gz".format(version),
                windows_msi = True,
                executable_paths = ["bin/tool.exe"],
            )
    """
    os = ctx["platform"]["os"]

    if os == "windows":
        if windows_url == None:
            return None
        if windows_msi:
            return msi_install(windows_url, executable_paths = executable_paths,
                               strip_prefix = strip_prefix)
        return archive_install(windows_url, strip_prefix = strip_prefix,
                               executable_paths = executable_paths)
    elif os == "macos":
        if macos_url == None:
            return None
        return archive_install(macos_url, strip_prefix = strip_prefix,
                               executable_paths = executable_paths)
    elif os == "linux":
        if linux_url == None:
            return None
        return archive_install(linux_url, strip_prefix = strip_prefix,
                               executable_paths = executable_paths)
    return None

# ---------------------------------------------------------------------------
# System tool finder descriptor (for prepare_execution)
# ---------------------------------------------------------------------------

def system_find(executable, system_paths = None, hint = None):
    """Return a system-find descriptor for the Rust runtime to locate a system tool.

    Used in `prepare_execution()` to find a tool that may already be installed
    on the system (via PATH or known system locations), before falling back to
    the vx-managed installation.

    This follows the same descriptor pattern as msi_install/archive_install:
    Starlark declares *what to look for*, the Rust runtime performs the actual
    filesystem search.

    Args:
        executable:   The executable name to search for (e.g. "7z", "git").
                      On Windows, ".exe" is appended automatically if not present.
        system_paths: Optional list of additional absolute paths to check
                      (e.g. ["C:\\Program Files\\7-Zip\\7z.exe"]).
                      These are checked after PATH lookup.
        hint:         Optional human-readable hint shown when the tool is not found
                      (e.g. "Install via: winget install 7zip.7zip").

    Returns:
        A system-find descriptor dict consumed by the Rust runtime.

    Example (in prepare_execution):
        def prepare_execution(ctx, version):
            return system_find(
                "7z",
                system_paths = [
                    "C:\\\\Program Files\\\\7-Zip\\\\7z.exe",
                    "C:\\\\Program Files (x86)\\\\7-Zip\\\\7z.exe",
                ],
                hint = "Install via: winget install 7zip.7zip",
            )
    """
    descriptor = {
        "__type":     "system_find",
        "executable": executable,
    }
    if system_paths != None:
        descriptor["system_paths"] = system_paths
    if hint != None:
        descriptor["hint"] = hint
    return descriptor

# ---------------------------------------------------------------------------
# Post-extract hook descriptor
# ---------------------------------------------------------------------------

def create_shim(name, target_executable, args = None, shim_dir = None):
    """Return a shim-creation descriptor for the Rust runtime to execute after extraction.

    Used in `post_extract()` to create wrapper scripts (shims) that forward
    calls to another executable with optional prepended arguments.

    This is the Starlark equivalent of `Shim::new(...).create(...)` in Rust.
    Follows the same descriptor pattern: Starlark declares *what to create*,
    Rust performs the actual filesystem write.

    Args:
        name:              Name of the shim to create (e.g. "bunx", "npx").
                           On Windows, ".cmd" is appended automatically.
        target_executable: Absolute or relative path to the target executable
                           that the shim wraps (e.g. "bun", "node").
        args:              Optional list of arguments to prepend when the shim
                           is invoked (e.g. ["x"] for `bun x`).
        shim_dir:          Optional directory where the shim is created.
                           If None, the shim is created in the same directory
                           as the target executable.

    Returns:
        A shim descriptor dict consumed by the Rust runtime.

    Example (in post_extract):
        def post_extract(ctx, version, install_dir):
            return [
                create_shim("bunx", "bun", args = ["x"]),
            ]
    """
    descriptor = {
        "__type": "create_shim",
        "name":   name,
        "target": target_executable,
    }
    if args != None:
        descriptor["args"] = args
    if shim_dir != None:
        descriptor["shim_dir"] = shim_dir
    return descriptor

def set_permissions(path, mode = "755"):
    """Return a permissions descriptor for the Rust runtime to apply after extraction.

    Used in `post_extract()` to set Unix file permissions on extracted files.
    On Windows, this is a no-op.

    Args:
        path: Relative path to the file within the install directory.
        mode: Unix permission mode string (default: "755").

    Returns:
        A permissions descriptor dict consumed by the Rust runtime.

    Example (in post_extract):
        def post_extract(ctx, version, install_dir):
            return [
                set_permissions("bin/mytool", "755"),
            ]
    """
    return {
        "__type": "set_permissions",
        "path":   path,
        "mode":   mode,
    }

# ---------------------------------------------------------------------------
# Pre-run hook descriptors
# ---------------------------------------------------------------------------

def ensure_dependencies(package_manager, check_file = "package.json",
                        lock_file = None, install_dir = "node_modules"):
    """Return a dependency-check descriptor for the Rust runtime to execute before running.

    Used in `pre_run()` to ensure project dependencies are installed before
    running a command. The Rust runtime checks if the install_dir exists and
    runs the package manager install command if not.

    This is the Starlark equivalent of `ensure_node_modules_installed()` in Rust.

    Args:
        package_manager: The package manager executable to run (e.g. "bun", "npm").
        check_file:      File that must exist for this check to apply
                         (e.g. "package.json"). If the file doesn't exist,
                         the check is skipped.
        lock_file:       Optional lock file to check (e.g. "bun.lockb").
                         If specified, the install is triggered when the lock
                         file is newer than install_dir.
        install_dir:     Directory to check for existence (default: "node_modules").
                         If this directory exists, the install is skipped.

    Returns:
        A dependency-check descriptor dict consumed by the Rust runtime.

    Example (in pre_run):
        def pre_run(ctx, args, executable):
            if len(args) > 0 and args[0] == "run":
                return [ensure_dependencies("bun")]
            return []
    """
    descriptor = {
        "__type":          "ensure_dependencies",
        "package_manager": package_manager,
        "check_file":      check_file,
        "install_dir":     install_dir,
    }
    if lock_file != None:
        descriptor["lock_file"] = lock_file
    return descriptor

def run_command(executable, args, working_dir = None, env = None,
                on_failure = "warn"):
    """Return a command-run descriptor for the Rust runtime to execute as a hook.

    Used in `post_extract()` or `pre_run()` to run arbitrary commands as part
    of the hook lifecycle. The Rust runtime executes the command and handles
    the result according to `on_failure`.

    Args:
        executable:  The executable to run (e.g. "chmod", "install_name_tool").
        args:        List of arguments to pass to the executable.
        working_dir: Optional working directory for the command.
                     If None, uses the current working directory.
        env:         Optional dict of environment variables to set.
        on_failure:  How to handle command failure:
                     - "warn":  Log a warning and continue (default)
                     - "error": Fail the hook with an error
                     - "ignore": Silently ignore failures

    Returns:
        A command descriptor dict consumed by the Rust runtime.

    Example (in post_extract):
        def post_extract(ctx, version, install_dir):
            if ctx["platform"]["os"] == "macos":
                return [
                    run_command("install_name_tool", ["-add_rpath", "@executable_path", "bin/mytool"]),
                ]
            return []
    """
    descriptor = {
        "__type":     "run_command",
        "executable": executable,
        "args":       args,
        "on_failure": on_failure,
    }
    if working_dir != None:
        descriptor["working_dir"] = working_dir
    if env != None:
        descriptor["env"] = env
    return descriptor

def flatten_dir(pattern = None, keep_subdirs = None):
    """Return a flatten-directory descriptor for the Rust runtime to apply after extraction.

    Used in `post_extract()` to flatten a nested directory structure into the
    install root. Many archives extract to a single top-level subdirectory
    (e.g. `jdk-21.0.1+12/`, `ffmpeg-7.1-essentials_build/`) — this descriptor
    instructs the Rust runtime to move all contents one level up and remove
    the now-empty subdirectory.

    This is the Starlark equivalent of the `post_extract()` directory-flattening
    logic found in Java, FFmpeg, and similar Rust runtimes.

    Args:
        pattern:      Optional glob pattern to match the subdirectory name
                      (e.g. "jdk-*", "ffmpeg-*"). If None, the Rust runtime
                      flattens the single subdirectory if exactly one exists.
        keep_subdirs: Optional list of subdirectory names to keep in place
                      rather than flattening (e.g. ["bin", "lib"]).
                      If None, all contents are moved up.

    Returns:
        A flatten-dir descriptor dict consumed by the Rust runtime.

    Example (in post_extract):
        def post_extract(ctx, version, install_dir):
            # Temurin archives extract to jdk-21.0.1+12/ — flatten it
            return [flatten_dir(pattern = "jdk-*")]

        def post_extract(ctx, version, install_dir):
            # FFmpeg Windows archives extract to ffmpeg-{version}-essentials_build/
            return [flatten_dir()]
    """
    descriptor = {"__type": "flatten_dir"}
    if pattern != None:
        descriptor["pattern"] = pattern
    if keep_subdirs != None:
        descriptor["keep_subdirs"] = keep_subdirs
    return descriptor
