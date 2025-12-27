#!/usr/bin/env python3
"""
Greet subcommand - greets someone by name.

Usage:
    vx x hello-world greet Alice
    vx x hello-world greet "World"
"""

import sys


def main():
    """Greet someone by name."""
    if len(sys.argv) < 2:
        print("Usage: vx x hello-world greet <name>")
        print("Example: vx x hello-world greet Alice")
        sys.exit(1)

    name = " ".join(sys.argv[1:])
    print(f"👋 Hello, {name}!")
    print(f"Welcome to the vx extension system!")


if __name__ == "__main__":
    main()
