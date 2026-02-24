# @vx//stdlib:system_install.star
# System package manager install helpers for vx provider scripts
#
# This module provides helpers for declaring system_install functions that
# delegate to OS package managers (winget, brew, apt, choco, etc.).
#
# ┌─────────────────────────────────────────────────────────────────────────┐
# │  pkg_strategy()           Single package manager strategy dict          │
# │  system_install_strategies() Wrap strategies into system_install result │
# │  winget_install()         Shorthand: winget strategy (Windows)          │
# │  choco_install()          Shorthand: Chocolatey strategy (Windows)      │
# │  scoop_install()          Shorthand: Scoop strategy (Windows)           │
# │  brew_install()           Shorthand: Homebrew strategy (macOS/Linux)    │
# │  apt_install()            Shorthand: APT strategy (Linux)               │
# │  dnf_install()            Shorthand: DNF strategy (Linux)               │
# │  pacman_install()         Shorthand: pacman strategy (Linux)            │
# │  snap_install()           Shorthand: snap strategy (Linux)              │
# │  cross_platform_install() system_install fn for cross-platform tools    │
# │  windows_install()        system_install fn for Windows-only tools      │
# │  multi_platform_install() system_install fn with full per-OS control    │
# └─────────────────────────────────────────────────────────────────────────┘

# ---------------------------------------------------------------------------
# Strategy builders
# ---------------------------------------------------------------------------

def pkg_strategy(manager, package, priority = 80, install_args = None,
                 platforms = None):
    """Build a single package manager install strategy dict.

    Use this as a building block for `system_install` functions, or pass
    a list of these to `system_install_strategies`.

    Args:
        manager:      Package manager name: "winget", "choco", "brew",
                      "apt", "dnf", "pacman", "scoop", "snap", "zypper"
        package:      Package identifier for this manager
        priority:     Install priority — higher = preferred (default: 80)
        install_args: Extra arguments to pass to the package manager
                      (e.g. "--add Microsoft.VisualStudio.Workload.VCTools")
        platforms:    OS list to restrict this strategy to
                      (e.g. ["windows"], ["linux", "macos"])
                      None = all platforms

    Returns:
        A strategy dict.

    Example:
        pkg_strategy("brew",   "ripgrep",    priority=90)
        pkg_strategy("winget", "BurntSushi.ripgrep", priority=85)
        pkg_strategy("apt",    "ripgrep",    priority=70)
    """
    s = {
        "manager":  manager,
        "package":  package,
        "priority": priority,
    }
    if install_args != None:
        s["install_args"] = install_args
    if platforms != None:
        s["platforms"] = platforms
    return s

def system_install_strategies(strategies):
    """Wrap a list of strategy dicts into the system_install return format.

    Args:
        strategies: List of strategy dicts (from pkg_strategy or hand-written)

    Returns:
        A system_install result dict: {"strategies": [...]}

    Example:
        def system_install(_ctx):
            return system_install_strategies([
                pkg_strategy("brew",   "ripgrep", priority=90),
                pkg_strategy("winget", "BurntSushi.ripgrep", priority=85),
                pkg_strategy("apt",    "ripgrep", priority=70),
            ])
    """
    return {"strategies": strategies}

# ---------------------------------------------------------------------------
# Per-manager shorthand builders
# ---------------------------------------------------------------------------

def winget_install(package, priority = 90, install_args = None):
    """Shorthand: single winget strategy (Windows only).

    Args:
        package:      winget package ID (e.g. "7zip.7zip")
        priority:     Install priority (default: 90)
        install_args: Extra winget arguments

    Returns:
        A strategy dict for winget.

    Example:
        winget_install("7zip.7zip")
        winget_install("Microsoft.VisualStudio.2022.BuildTools",
                       install_args="--add Microsoft.VisualStudio.Workload.VCTools")
    """
    return pkg_strategy("winget", package, priority = priority,
                        install_args = install_args, platforms = ["windows"])

def choco_install(package, priority = 80, install_args = None):
    """Shorthand: single Chocolatey strategy (Windows only).

    Args:
        package:      Chocolatey package name (e.g. "7zip")
        priority:     Install priority (default: 80)
        install_args: Extra choco arguments

    Returns:
        A strategy dict for choco.

    Example:
        choco_install("7zip")
    """
    return pkg_strategy("choco", package, priority = priority,
                        install_args = install_args, platforms = ["windows"])

def scoop_install(package, priority = 70):
    """Shorthand: single Scoop strategy (Windows only).

    Args:
        package:  Scoop package name (e.g. "git")
        priority: Install priority (default: 70)

    Returns:
        A strategy dict for scoop.

    Example:
        scoop_install("git")
    """
    return pkg_strategy("scoop", package, priority = priority,
                        platforms = ["windows"])

def brew_install(package, priority = 90):
    """Shorthand: single Homebrew strategy (macOS/Linux).

    Args:
        package:  Homebrew formula or cask name (e.g. "ripgrep", "sevenzip")
        priority: Install priority (default: 90)

    Returns:
        A strategy dict for brew.

    Example:
        brew_install("ripgrep")
        brew_install("sevenzip")
    """
    return pkg_strategy("brew", package, priority = priority)

def apt_install(package, priority = 80):
    """Shorthand: single APT strategy (Linux only).

    Args:
        package:  APT package name (e.g. "ripgrep", "p7zip-full")
        priority: Install priority (default: 80)

    Returns:
        A strategy dict for apt.

    Example:
        apt_install("ripgrep")
        apt_install("p7zip-full")
    """
    return pkg_strategy("apt", package, priority = priority,
                        platforms = ["linux"])

def dnf_install(package, priority = 75):
    """Shorthand: single DNF/YUM strategy (Linux only).

    Args:
        package:  DNF package name (e.g. "ripgrep")
        priority: Install priority (default: 75)

    Returns:
        A strategy dict for dnf.

    Example:
        dnf_install("ripgrep")
    """
    return pkg_strategy("dnf", package, priority = priority,
                        platforms = ["linux"])

def pacman_install(package, priority = 70):
    """Shorthand: single pacman strategy (Linux only).

    Args:
        package:  pacman package name (e.g. "ripgrep")
        priority: Install priority (default: 70)

    Returns:
        A strategy dict for pacman.

    Example:
        pacman_install("ripgrep")
    """
    return pkg_strategy("pacman", package, priority = priority,
                        platforms = ["linux"])

def snap_install(package, priority = 60, classic = False):
    """Shorthand: single snap strategy (Linux only).

    Args:
        package:  snap package name (e.g. "ripgrep")
        priority: Install priority (default: 60)
        classic:  Use --classic confinement (default: False)

    Returns:
        A strategy dict for snap.

    Example:
        snap_install("ripgrep")
        snap_install("code", classic=True)
    """
    args = "--classic" if classic else None
    return pkg_strategy("snap", package, priority = priority,
                        install_args = args, platforms = ["linux"])

# ---------------------------------------------------------------------------
# High-level system_install function builders
# ---------------------------------------------------------------------------

def cross_platform_install(windows = None, macos = None, linux = None,
                           windows_priority = 90, macos_priority = 90,
                           linux_priority = 80):
    """Build a system_install function for tools available on all platforms.

    This is the most common pattern: different package names/managers per OS.
    Returns a ready-to-use `system_install` function.

    Args:
        windows:          winget package ID for Windows (or None to skip)
        macos:            brew formula for macOS (or None to skip)
        linux:            apt package name for Linux (or None to skip)
        windows_priority: Priority for the winget strategy (default: 90)
        macos_priority:   Priority for the brew strategy (default: 90)
        linux_priority:   Priority for the apt strategy (default: 80)

    Returns:
        A function: system_install(ctx) -> dict

    Example:
        # nasm: same package name everywhere
        system_install = cross_platform_install(
            windows = "NASM.NASM",
            macos   = "nasm",
            linux   = "nasm",
        )

        # 7zip: different package names
        system_install = cross_platform_install(
            windows = "7zip.7zip",
            macos   = "sevenzip",
            linux   = "p7zip-full",
        )

        # macOS + Linux only (no Windows package)
        system_install = cross_platform_install(
            macos = "bash",
            linux = "bash",
        )
    """
    def _system_install(ctx):
        strategies = []
        os = ctx.platform.os
        if os == "windows" and windows != None:
            strategies.append(winget_install(windows, priority = windows_priority))
        elif os == "macos" and macos != None:
            strategies.append(brew_install(macos, priority = macos_priority))
        elif os == "linux":
            if linux != None:
                strategies.append(apt_install(linux, priority = linux_priority))
            if macos != None:
                # brew is also available on Linux
                strategies.append(brew_install(macos, priority = linux_priority - 10))
        if not strategies:
            return {}
        return system_install_strategies(strategies)
    return _system_install

def windows_install(winget = None, choco = None, scoop = None,
                    winget_priority = 90, choco_priority = 80,
                    scoop_priority = 70):
    """Build a system_install function for Windows-only tools.

    Returns a ready-to-use `system_install` function that only applies on Windows.

    Args:
        winget:           winget package ID (or None to skip)
        choco:            Chocolatey package name (or None to skip)
        scoop:            Scoop package name (or None to skip)
        winget_priority:  Priority for winget (default: 90)
        choco_priority:   Priority for choco (default: 80)
        scoop_priority:   Priority for scoop (default: 70)

    Returns:
        A function: system_install(ctx) -> dict

    Example:
        # nuget: Windows only
        system_install = windows_install(
            winget = "Microsoft.NuGet",
            choco  = "nuget.commandline",
        )

        # rcedit: Windows only, choco only
        system_install = windows_install(choco="rcedit")
    """
    def _system_install(ctx):
        if ctx.platform.os != "windows":
            return {}
        strategies = []
        if winget != None:
            strategies.append(winget_install(winget, priority = winget_priority))
        if choco != None:
            strategies.append(choco_install(choco, priority = choco_priority))
        if scoop != None:
            strategies.append(scoop_install(scoop, priority = scoop_priority))
        if not strategies:
            return {}
        return system_install_strategies(strategies)
    return _system_install

def multi_platform_install(windows_strategies = None, macos_strategies = None,
                           linux_strategies = None):
    """Build a system_install function with full per-platform strategy lists.

    Use this when you need multiple package managers per platform
    (e.g. both winget and choco on Windows, both apt and dnf on Linux).

    Args:
        windows_strategies: List of strategy dicts for Windows (or None)
        macos_strategies:   List of strategy dicts for macOS (or None)
        linux_strategies:   List of strategy dicts for Linux (or None)

    Returns:
        A function: system_install(ctx) -> dict

    Example:
        # bash: winget+choco+scoop on Windows, brew on macOS, apt+dnf+pacman on Linux
        system_install = multi_platform_install(
            windows_strategies = [
                winget_install("Git.Git",  priority=95),
                choco_install("git",       priority=80),
                scoop_install("git",       priority=60),
            ],
            macos_strategies = [
                brew_install("bash", priority=90),
            ],
            linux_strategies = [
                apt_install("bash",    priority=90),
                dnf_install("bash",    priority=85),
                pacman_install("bash", priority=80),
            ],
        )
    """
    def _system_install(ctx):
        os = ctx.platform.os
        if os == "windows" and windows_strategies != None:
            return system_install_strategies(windows_strategies)
        elif os == "macos" and macos_strategies != None:
            return system_install_strategies(macos_strategies)
        elif os == "linux" and linux_strategies != None:
            return system_install_strategies(linux_strategies)
        return {}
    return _system_install
