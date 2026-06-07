#!/bin/bash
set -e

echo "CrossCrypt Dependency Installer"
echo "================================"

# Detect OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if command -v apt-get &> /dev/null; then
        echo "Installing dependencies for Debian/Ubuntu..."
        sudo apt-get update
        sudo apt-get install -y \
            build-essential \
            pkg-config \
            libfuse3-dev \
            libssl-dev
    elif command -v yum &> /dev/null; then
        echo "Installing dependencies for RHEL/CentOS..."
        sudo yum groupinstall -y "Development Tools"
        sudo yum install -y \
            pkgconfig \
            fuse3-devel \
            openssl-devel
    elif command -v pacman &> /dev/null; then
        echo "Installing dependencies for Arch Linux..."
        sudo pacman -S --needed \
                base-devel \
                pkgconf \
                fuse3 \
                openssl
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Installing dependencies for macOS..."
    if ! command -v brew &> /dev/null; then
        echo "Homebrew not found. Please install from https://brew.sh"
        exit 1
    fi
    brew install \
        pkgconf \
        macfuse
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    echo "Windows dependencies:"
    echo "1. Install Visual Studio Build Tools"
    echo "2. Install WinFsp from https://winfsp.dev"
    echo "3. Install Rust from https://rustup.rs"
fi

echo ""
echo "Dependencies installed successfully!"
