# @vx//stdlib:runtime.star
# Runtime definition builders for vx provider scripts
#
# This module provides helpers for declaring runtimes and their dependencies
# inside a provider.star file.
#
# ┌─────────────────────────────────────────────────────────────────────────┐
# │  runtime_def()          Single-executable runtime definition            │
# │  bundled_runtime_def()  Runtime bundled inside another (npm, gofmt…)   │
# │  dep_def()              Runtime dependency declaration                  │
# └─────────────────────────────────────────────────────────────────────────┘

# ---------------------------------------------------------------------------
# runtime_def — runtime definition builder
# ---------------------------------------------------------------------------

def runtime_def(name, executable = None, description = None, aliases = None,
                priority = 100, version_cmd = None, version_pattern = None,
                test_commands = None):
    """Build a runtime definition dict for use in the `runtimes` list.

    Covers the common case of a single-executable tool with a `--version` check.
    For complex multi-executable runtimes, write the dict directly.

    Args:
        name:            Runtime name (e.g. "mytool")
        executable:      Executable name; defaults to `name`
        description:     Human-readable description; defaults to `name`
        aliases:         List of alias strings (default: [])
        priority:        Install priority (default: 100)
        version_cmd:     Version check command template
                         (default: "{executable} --version")
        version_pattern: Expected output regex (default: None)
        test_commands:   Full test_commands list; overrides version_cmd/version_pattern

    Returns:
        A runtime definition dict.

    Example:
        runtimes = [runtime_def("rg", executable="rg", aliases=["ripgrep"])]
        runtimes = [runtime_def("jj", version_pattern="jj \\d+")]
    """
    exe  = executable   if executable   != None else name
    desc = description  if description  != None else name
    also  = aliases      if aliases      != None else []

    if test_commands != None:
        cmds = test_commands
    else:
        cmd = version_cmd if version_cmd != None else "{executable} --version"
        if version_pattern != None:
            cmds = [{"command": cmd, "name": "version_check",
                     "expected_output": version_pattern}]
        else:
            cmds = [{"command": cmd, "name": "version_check"}]

    return {
        "name":          name,
        "executable":    exe,
        "description":   desc,
        "aliases":       also,
        "priority":      priority,
        "test_commands": cmds,
    }

# ---------------------------------------------------------------------------
# bundled_runtime_def — runtime definition for tools bundled with another
# ---------------------------------------------------------------------------

def bundled_runtime_def(name, bundled_with, executable = None, description = None,
                        aliases = None, command_prefix = None, test_commands = None,
                        version_pattern = None):
    """Build a runtime definition for a tool bundled inside another tool's install.

    Use this for runtimes that are shipped as part of another tool's archive
    (e.g. npm/npx bundled with Node.js, gofmt bundled with Go, javac with JDK).
    The Rust runtime will look for this executable inside the primary tool's
    install directory rather than downloading it separately.

    Args:
        name:           Runtime name (e.g. "npm", "gofmt", "javac")
        bundled_with:   Name of the primary runtime that ships this tool
                        (e.g. "node", "go", "java")
        executable:     Executable name; defaults to `name`
        description:    Human-readable description
        aliases:        List of alias strings (default: [])
        command_prefix: List of args to prepend when invoking the executable.
                        e.g. ["x"] makes `bunx foo` invoke `bun x foo`
        test_commands:  Full test_commands list; overrides version_pattern
        version_pattern: Expected output regex for the default --version check

    Returns:
        A runtime definition dict with "bundled_with" set.

    Example:
        runtimes = [
            runtime_def("node"),
            bundled_runtime_def("npm",  bundled_with="node"),
            bundled_runtime_def("npx",  bundled_with="node"),
            bundled_runtime_def("pip",  bundled_with="python", aliases=["pip3"]),
            bundled_runtime_def("bunx", bundled_with="bun",
                                executable="bun", command_prefix=["x"]),
        ]
    """
    exe  = executable  if executable  != None else name
    desc = description if description != None else "{} (bundled with {})".format(name, bundled_with)
    also = aliases     if aliases     != None else []

    if test_commands != None:
        cmds = test_commands
    elif version_pattern != None:
        cmds = [{"command": "{executable} --version", "name": "version_check",
                 "expected_output": version_pattern}]
    else:
        cmds = [{"command": "{executable} --version", "name": "version_check"}]

    entry = {
        "name":          name,
        "executable":    exe,
        "description":   desc,
        "aliases":       also,
        "bundled_with":  bundled_with,
        "test_commands": cmds,
    }
    if command_prefix != None:
        entry["command_prefix"] = command_prefix
    return entry

# ---------------------------------------------------------------------------
# dep_def — runtime dependency declaration
# ---------------------------------------------------------------------------

def dep_def(runtime, version = "*", optional = False, reason = None):
    """Build a dependency declaration for use in the `deps` function.

    Use this to declare that a provider requires or recommends another runtime.
    The Rust resolver uses these declarations to auto-install dependencies.

    Args:
        runtime:  Name of the required runtime (e.g. "git", "node")
        version:  Version constraint (default: "*" = any version)
                  Supports semver ranges: ">=18", "^20", "~1.21"
        optional: If True, the dependency is recommended but not required.
                  vx will warn but not fail if it's missing (default: False)
        reason:   Human-readable explanation shown to the user

    Returns:
        A dependency dict for use in the deps() return list.

    Example:
        def deps(_ctx, _version):
            return [
                dep_def("git", optional=True,
                        reason="Git is required for fetching Go modules"),
                dep_def("node", version=">=18",
                        reason="Requires Node.js 18+"),
            ]
    """
    entry = {
        "runtime":  runtime,
        "version":  version,
        "optional": optional,
    }
    if reason != None:
        entry["reason"] = reason
    return entry
