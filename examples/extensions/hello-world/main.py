#!/usr/bin/env python3
"""
Hello World Extension - Main Entry Point

This is a simple example extension demonstrating vx extension capabilities.

Usage:
    vx x hello-world              # Run main entry point
    vx x hello-world greet Alice  # Run greet subcommand
    vx x hello-world info         # Show extension info
"""

import os
import sys


def main():
    """Main entry point for the hello-world extension."""
    print("👋 Hello from vx extension!")
    print()
    print("This is the main entry point of the hello-world extension.")
    print()
    print("Available commands:")
    print("  vx x hello-world greet <name>  - Greet someone")
    print("  vx x hello-world info          - Show extension info")
    print()
    print("Environment variables available:")
    print(f"  VX_VERSION:        {os.environ.get('VX_VERSION', 'N/A')}")
    print(f"  VX_EXTENSION_NAME: {os.environ.get('VX_EXTENSION_NAME', 'N/A')}")
    print(f"  VX_EXTENSION_DIR:  {os.environ.get('VX_EXTENSION_DIR', 'N/A')}")
    print(f"  VX_PROJECT_DIR:    {os.environ.get('VX_PROJECT_DIR', 'N/A')}")
    print(f"  VX_HOME:           {os.environ.get('VX_HOME', 'N/A')}")


if __name__ == "__main__":
    main()
