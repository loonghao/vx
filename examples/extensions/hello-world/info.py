#!/usr/bin/env python3
"""
Info subcommand - shows extension information.

Usage:
    vx x hello-world info
"""

import os
import sys
import platform


def main():
    """Show extension and environment information."""
    print("=" * 50)
    print("Hello World Extension - Info")
    print("=" * 50)
    print()

    # Extension info
    print("📦 Extension Information:")
    print(f"  Name:    {os.environ.get('VX_EXTENSION_NAME', 'N/A')}")
    print(f"  Dir:     {os.environ.get('VX_EXTENSION_DIR', 'N/A')}")
    print()

    # VX info
    print("🔧 VX Information:")
    print(f"  Version: {os.environ.get('VX_VERSION', 'N/A')}")
    print(f"  Home:    {os.environ.get('VX_HOME', 'N/A')}")
    print(f"  Runtimes:{os.environ.get('VX_RUNTIMES_DIR', 'N/A')}")
    print()

    # Project info
    print("📁 Project Information:")
    print(f"  Dir:     {os.environ.get('VX_PROJECT_DIR', 'N/A')}")
    print()

    # Runtime info
    print("🐍 Python Runtime:")
    print(f"  Version: {platform.python_version()}")
    print(f"  Path:    {sys.executable}")
    print()

    # System info
    print("💻 System Information:")
    print(f"  OS:      {platform.system()} {platform.release()}")
    print(f"  Arch:    {platform.machine()}")
    print()


if __name__ == "__main__":
    main()
