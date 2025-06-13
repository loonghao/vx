#!/bin/bash
# Shell script to install LLVM on Unix systems
# This script is used by GoReleaser for PGO builds

set -e

echo "Installing LLVM for PGO builds..."

# Detect the operating system
if command -v apt-get >/dev/null 2>&1; then
    echo "Installing LLVM via apt-get (Debian/Ubuntu)..."
    sudo apt-get update && sudo apt-get install -y llvm
elif command -v brew >/dev/null 2>&1; then
    echo "Installing LLVM via Homebrew (macOS)..."
    brew install llvm
elif command -v yum >/dev/null 2>&1; then
    echo "Installing LLVM via yum (RHEL/CentOS)..."
    sudo yum install -y llvm
elif command -v dnf >/dev/null 2>&1; then
    echo "Installing LLVM via dnf (Fedora)..."
    sudo dnf install -y llvm
elif command -v pacman >/dev/null 2>&1; then
    echo "Installing LLVM via pacman (Arch Linux)..."
    sudo pacman -S --noconfirm llvm
elif command -v apk >/dev/null 2>&1; then
    echo "Installing LLVM via apk (Alpine Linux)..."
    sudo apk add --no-cache llvm
else
    echo "Error: No supported package manager found."
    echo "Please install LLVM manually for your system."
    echo "Visit: https://releases.llvm.org/"
    exit 1
fi

echo "LLVM installation completed."
