# Installation Guide

## Pre-built Binaries

Download the latest release for your platform:

- **macOS (Intel)**: `crosscrypt-macos-x86_64.tar.gz`
- **macOS (Apple Silicon)**: `crosscrypt-macos-arm64.tar.gz`
- **Linux**: `crosscrypt-linux-x86_64.tar.gz`
- **Windows**: `crosscrypt-windows-x86_64.zip`

### macOS

```bash
# Download and extract
tar xzf crosscrypt-macos-x86_64.tar.gz

# Run
./crosscrypt-macos-x86_64/crosscrypt --help
```

### Linux

```bash
# Download and extract
tar xzf crosscrypt-linux-x86_64.tar.gz

# Install dependencies (Ubuntu/Debian)
sudo apt-get install libfuse3-dev

# Run
./crosscrypt-linux-x86_64/crosscrypt --help
```

### Windows

```powershell
# Extract zip
Expand-Archive crosscrypt-windows-x86_64.zip

# Run
.\crosscrypt-windows-x86_64\crosscrypt.exe --help
```

## Build from Source

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs)
- Platform-specific dependencies:
  - **Windows**: Visual Studio Build Tools
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `libfuse3-dev`, `pkg-config`

### Build

```bash
# Clone repository
git clone https://github.com/crosscrypt/crosscrypt.git
cd crosscrypt

# Build release binary
cargo build --release

# Binary location:
# - Linux/macOS: target/release/crosscrypt
# - Windows: target/release/crosscrypt.exe
```

### Cross-compilation

```bash
# Windows from macOS/Linux
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

# macOS ARM from macOS Intel
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

## Usage

```bash
# Create encrypted volume
crosscrypt create -d /dev/sdb -l "My Drive"

# Mount
crosscrypt mount -d /dev/sdb

# Unmount
crosscrypt unmount -d /dev/sdb

# Emergency lock
crosscrypt lock -d /dev/sdb
```

## Portable Deployment

Copy the binary to your USB drive:

```bash
# macOS
cp target/release/crosscrypt /Volumes/MyUSB/

# Linux
cp target/release/crosscrypt /media/myusb/

# Windows
copy target\release\crosscrypt.exe E:\
```

The binary runs directly without installation!
