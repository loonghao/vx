# @vx//stdlib:env.star — Environment variable operation helpers
#
# Provides rez-style environment variable operations for provider.star scripts.
# The `environment(ctx, version)` function should return a list of these ops.
#
# The Rust runtime applies them in order, enabling layered composition when
# multiple providers contribute to the same environment (e.g. PATH).
#
# Usage:
#   load("@vx//stdlib:env.star", "env_set", "env_prepend", "env_append", "env_unset")
#
#   def environment(ctx, version):
#       return [
#           env_set("GOROOT", ctx.install_dir),
#           env_prepend("PATH", ctx.install_dir + "/bin"),
#           env_set("GO111MODULE", "on"),
#       ]
#
# Separator defaults:
#   - Windows: ";"
#   - Unix:    ":"
#
# The Rust side reads the "op" field to dispatch to the correct operation.

def env_set(key, value):
    """Set an environment variable to a fixed value (overwrite any existing value).

    Args:
        key:   Environment variable name (e.g. "GOROOT")
        value: Value to set

    Returns:
        An EnvOp dict with op="set"
    """
    return {"op": "set", "key": key, "value": value}

def env_prepend(key, value, sep = None):
    """Prepend a value to an environment variable (PATH-style).

    If the variable already exists, the new value is prepended with `sep`.
    If the variable does not exist, it is set to `value`.

    Args:
        key:   Environment variable name (e.g. "PATH")
        value: Value to prepend (e.g. "/usr/local/bin")
        sep:   Separator (default: ":" on Unix, ";" on Windows)

    Returns:
        An EnvOp dict with op="prepend"
    """
    op = {"op": "prepend", "key": key, "value": value}
    if sep != None:
        op["sep"] = sep
    return op

def env_append(key, value, sep = None):
    """Append a value to an environment variable (PATH-style).

    If the variable already exists, the new value is appended with `sep`.
    If the variable does not exist, it is set to `value`.

    Args:
        key:   Environment variable name (e.g. "PATH")
        value: Value to append (e.g. "/usr/local/bin")
        sep:   Separator (default: ":" on Unix, ";" on Windows)

    Returns:
        An EnvOp dict with op="append"
    """
    op = {"op": "append", "key": key, "value": value}
    if sep != None:
        op["sep"] = sep
    return op

def env_unset(key):
    """Remove an environment variable.

    Args:
        key: Environment variable name to remove

    Returns:
        An EnvOp dict with op="unset"
    """
    return {"op": "unset", "key": key}
