# @vx//stdlib:provider.star
# High-level provider helpers for vx provider scripts — re-export facade
#
# This module re-exports all provider helpers from their dedicated sub-modules.
# Import from here for convenience, or import directly from the sub-module
# when you only need a specific group of helpers.
#
# Sub-modules (single-responsibility):
#
#   @vx//stdlib:runtime.star         — runtime_def, bundled_runtime_def, dep_def
#   @vx//stdlib:platform.star        — platform_map, platform_select (+ is_windows, …)
#   @vx//stdlib:permissions.star     — github_permissions, system_permissions
#   @vx//stdlib:layout.star          — archive_layout, binary_layout, bin_subdir_layout,
#                                      post_extract_*, pre_run_ensure_deps,
#                                      bin_subdir_env, bin_subdir_execute_path,
#                                      path_fns, path_env_fns,
#                                      fetch_versions_from_api, fetch_versions_with_tag_prefix
#   @vx//stdlib:system_install.star  — pkg_strategy, system_install_strategies,
#                                      winget_install, choco_install, scoop_install,
#                                      brew_install, apt_install, dnf_install,
#                                      pacman_install, snap_install,
#                                      cross_platform_install, windows_install,
#                                      multi_platform_install
#   @vx//stdlib:script_install.star  — curl_bash_install, curl_sh_install,
#                                      irm_iex_install, irm_install,
#                                      platform_script_install
#   @vx//stdlib:provider_templates.star — github_rust_provider, github_go_provider,
#                                         github_binary_provider, system_provider
#
# ┌─────────────────────────────────────────────────────────────────────────┐
# │  Provider templates (Level 1 — zero boilerplate)                        │
# │                                                                         │
# │  github_rust_provider()   GitHub releases, Rust target triple naming    │
# │  github_go_provider()     GitHub releases, Go-style os/arch naming      │
# │  github_binary_provider() GitHub releases, single binary (no archive)   │
# │  system_provider()        System package manager (winget/brew/apt)      │
# ├─────────────────────────────────────────────────────────────────────────┤
# │  Runtime / permission builders                                          │
# │                                                                         │
# │  runtime_def()            Single-executable runtime definition          │
# │  bundled_runtime_def()    Runtime bundled inside another (npm, gofmt…)  │
# │  dep_def()                Runtime dependency declaration                │
# │  github_permissions()     Permissions for GitHub-hosted tools           │
# │  system_permissions()     Permissions for system package manager tools  │
# ├─────────────────────────────────────────────────────────────────────────┤
# │  Hook builders (Level 2 — custom logic, standard hooks)                 │
# │                                                                         │
# │  post_extract_flatten()   Flatten versioned top-level dir after extract │
# │  post_extract_shim()      Create a shim executable after extract        │
# │  post_extract_permissions() Set Unix execute permissions after extract  │
# │  post_extract_combine()   Combine multiple post_extract hooks           │
# │  pre_run_ensure_deps()    Ensure project deps before running a command  │
# ├─────────────────────────────────────────────────────────────────────────┤
# │  fetch_versions helpers                                                 │
# │                                                                         │
# │  fetch_versions_from_api()        Non-GitHub JSON API (nodejs.org, …)  │
# │  fetch_versions_with_tag_prefix() Non-standard tag prefix (bun-v…)     │
# ├─────────────────────────────────────────────────────────────────────────┤
# │  Layout / path helpers                                                  │
# │                                                                         │
# │  archive_layout()         Standalone archive install_layout builder     │
# │  binary_layout()          Standalone binary install_layout builder      │
# │  bin_subdir_layout()      Layout for tools with bin/ subdir (node/go)   │
# │  bin_subdir_env()         PATH env for tools with bin/ subdir           │
# │  bin_subdir_execute_path() get_execute_path for bin/ subdir tools       │
# │  path_fns()               store_root + get_execute_path helpers         │
# │  path_env_fns()           environment + post_install helpers            │
# ├─────────────────────────────────────────────────────────────────────────┤
# │  System install helpers (for system_install functions)                  │
# │                                                                         │
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
# ├─────────────────────────────────────────────────────────────────────────┤
# │  Script install helpers (for script_install functions)                  │
# │                                                                         │
# │  curl_bash_install()      curl | bash pattern (Unix)                    │
# │  curl_sh_install()        curl | sh pattern (Unix, POSIX)               │
# │  irm_iex_install()        PowerShell iex(irm) pattern (Windows)         │
# │  irm_install()            PowerShell irm | iex modern pattern (Windows) │
# │  platform_script_install() Dispatch unix/windows script by OS           │
# ├─────────────────────────────────────────────────────────────────────────┤
# │  Platform helpers                                                       │
# │                                                                         │
# │  platform_map()           Look up value from {os}/{arch} keyed dict     │
# │  platform_select()        Select value by OS (windows/macos/linux)      │
# └─────────────────────────────────────────────────────────────────────────┘

# ---------------------------------------------------------------------------
# Re-exports from sub-modules
# ---------------------------------------------------------------------------

load("@vx//stdlib:runtime.star",
     "runtime_def",
     "bundled_runtime_def",
     "dep_def")

load("@vx//stdlib:platform.star",
     "platform_map",
     "platform_select")

load("@vx//stdlib:permissions.star",
     "github_permissions",
     "system_permissions")

load("@vx//stdlib:layout.star",
     "archive_layout",
     "binary_layout",
     "bin_subdir_layout",
     "post_extract_flatten",
     "post_extract_shim",
     "post_extract_permissions",
     "post_extract_combine",
     "pre_run_ensure_deps",
     "fetch_versions_from_api",
     "fetch_versions_with_tag_prefix",
     "bin_subdir_env",
     "bin_subdir_execute_path",
     "path_fns",
     "path_env_fns")

load("@vx//stdlib:system_install.star",
     "pkg_strategy",
     "system_install_strategies",
     "winget_install",
     "choco_install",
     "scoop_install",
     "brew_install",
     "apt_install",
     "dnf_install",
     "pacman_install",
     "snap_install",
     "cross_platform_install",
     "windows_install",
     "multi_platform_install")

load("@vx//stdlib:script_install.star",
     "curl_bash_install",
     "curl_sh_install",
     "irm_iex_install",
     "irm_install",
     "platform_script_install")

load("@vx//stdlib:provider_templates.star",
     "github_rust_provider",
     "github_go_provider",
     "github_binary_provider",
     "system_provider")
