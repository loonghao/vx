# @vx//stdlib:test.star
#
# Functional test DSL for provider.star test_commands.
#
# This module provides helper functions to build structured test command
# descriptors that the vx test framework understands.  Import it in your
# provider.star and use the helpers to declare rich test suites:
#
#   load("@vx//stdlib:test.star",
#        "cmd", "check_path", "check_env", "check_file",
#        "check_not_path", "check_not_env")
#
#   runtimes = [
#       {
#           "name": "mytool",
#           "executable": "mytool",
#           "test_commands": [
#               cmd("{executable} --version", name="version_check",
#                   expected_output="^\\d+\\.\\d+"),
#               check_path("{install_dir}/bin/mytool",
#                          name="binary_exists"),
#               check_env("MYTOOL_HOME",
#                         name="env_set",
#                         expected_output=".*mytool.*"),
#               check_file("{install_dir}/VERSION",
#                          name="version_file",
#                          expected_output="\\d+\\.\\d+"),
#           ],
#       },
#   ]

# ---------------------------------------------------------------------------
# Command check — run a shell command and inspect exit code / output
# ---------------------------------------------------------------------------

def cmd(command, name=None, expect_success=True, expected_output=None, timeout_ms=None):
    """Run a shell command and check its exit code and/or output.

    Args:
        command:         Command template.  Supports {executable}, {install_dir},
                         {vx_home} substitutions.
        name:            Human-readable test name (shown in test output).
        expect_success:  If True (default), the command must exit with code 0.
        expected_output: Optional regex pattern that must match stdout or stderr.
        timeout_ms:      Per-command timeout in milliseconds (overrides global).

    Returns:
        A test command descriptor dict.
    """
    entry = {
        "command":    command,
        "check_type": "command",
        "expect_success": expect_success,
    }
    if name != None:
        entry["name"] = name
    if expected_output != None:
        entry["expected_output"] = expected_output
    if timeout_ms != None:
        entry["timeout_ms"] = timeout_ms
    return entry


# ---------------------------------------------------------------------------
# Path checks — assert filesystem paths exist (or not)
# ---------------------------------------------------------------------------

def check_path(path, name=None):
    """Assert that a file or directory exists at the given path.

    Args:
        path:  Path template.  Supports {executable}, {install_dir}, {vx_home}.
        name:  Human-readable test name.

    Returns:
        A test command descriptor dict.
    """
    entry = {
        "command":    path,
        "check_type": "check_path",
    }
    if name != None:
        entry["name"] = name
    return entry


def check_not_path(path, name=None):
    """Assert that a path does NOT exist.

    Args:
        path:  Path template.  Supports {executable}, {install_dir}, {vx_home}.
        name:  Human-readable test name.

    Returns:
        A test command descriptor dict.
    """
    entry = {
        "command":    path,
        "check_type": "check_not_path",
    }
    if name != None:
        entry["name"] = name
    return entry


# ---------------------------------------------------------------------------
# Environment variable checks
# ---------------------------------------------------------------------------

def check_env(var_name, name=None, expected_output=None):
    """Assert that an environment variable is set.

    Args:
        var_name:        Name of the environment variable to check.
        name:            Human-readable test name.
        expected_output: Optional regex pattern that the variable value must match.

    Returns:
        A test command descriptor dict.
    """
    entry = {
        "command":    var_name,
        "check_type": "check_env",
    }
    if name != None:
        entry["name"] = name
    if expected_output != None:
        entry["expected_output"] = expected_output
    return entry


def check_not_env(var_name, name=None):
    """Assert that an environment variable is NOT set.

    Args:
        var_name:  Name of the environment variable to check.
        name:      Human-readable test name.

    Returns:
        A test command descriptor dict.
    """
    entry = {
        "command":    var_name,
        "check_type": "check_not_env",
    }
    if name != None:
        entry["name"] = name
    return entry


# ---------------------------------------------------------------------------
# File content checks
# ---------------------------------------------------------------------------

def check_file(path, name=None, expected_output=None):
    """Assert that a file exists and optionally that its content matches a pattern.

    Args:
        path:            File path template.  Supports {executable}, {install_dir},
                         {vx_home}.
        name:            Human-readable test name.
        expected_output: Optional regex pattern that the file content must match.

    Returns:
        A test command descriptor dict.
    """
    entry = {
        "command":    path,
        "check_type": "check_file",
    }
    if name != None:
        entry["name"] = name
    if expected_output != None:
        entry["expected_output"] = expected_output
    return entry
