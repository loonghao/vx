# @vx//stdlib:permissions.star
# Permission builders for vx provider scripts
#
# This module provides helpers for declaring network and execution permissions
# required by a provider.
#
# ┌─────────────────────────────────────────────────────────────────────────┐
# │  github_permissions()   Permissions for GitHub-hosted tools             │
# │  system_permissions()   Permissions for system package manager tools    │
# └─────────────────────────────────────────────────────────────────────────┘

# ---------------------------------------------------------------------------
# Permissions builders
# ---------------------------------------------------------------------------

def github_permissions(extra_hosts = None, exec_cmds = None):
    """Permissions dict for tools downloaded from GitHub releases.

    Pre-allows api.github.com and github.com. Pass `extra_hosts` for tools
    that also download from a CDN or custom domain (e.g. Helm, kubectl).

    Args:
        extra_hosts: Additional HTTP hosts to allow (list of strings).
                     e.g. ["get.helm.sh", "dl.k8s.io"]
        exec_cmds:   Executable names allowed to run (list of strings).
                     e.g. ["winget", "brew", "apt"]

    Returns:
        A permissions dict.

    Example:
        permissions = github_permissions()
        permissions = github_permissions(extra_hosts=["get.helm.sh"])
        permissions = github_permissions(exec_cmds=["winget", "brew"])
    """
    hosts = ["api.github.com", "github.com"]
    if extra_hosts != None:
        hosts = hosts + extra_hosts
    return {
        "http": hosts,
        "fs":   [],
        "exec": exec_cmds if exec_cmds != None else [],
    }

def system_permissions(exec_cmds = None, extra_hosts = None):
    """Permissions dict for system tools that use package managers.

    No HTTP hosts are pre-allowed (system tools don't download from GitHub).

    Args:
        exec_cmds:   Package manager executables to allow.
                     e.g. ["winget", "choco", "brew", "apt"]
        extra_hosts: HTTP hosts to allow (list of strings).

    Returns:
        A permissions dict.

    Example:
        permissions = system_permissions(exec_cmds=["winget", "brew", "apt"])
    """
    return {
        "http": extra_hosts if extra_hosts != None else [],
        "fs":   [],
        "exec": exec_cmds  if exec_cmds  != None else [],
    }
