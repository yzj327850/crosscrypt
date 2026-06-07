# CrossCrypt

Cross-platform portable disk encryption solution.

## Features

- 🔒 **Military-grade encryption** - AES-256-XTS with Argon2id key derivation
- 💻 **Cross-platform** - Windows, macOS, Linux support
- 🔑 **Portable** - Runs directly from USB drive, no installation needed
- 🛡️ **Brute force protection** - Auto-lock and secure wipe after failed attempts
- 📁 **In-place encryption** - Encrypt existing data without formatting
- ⚡ **High performance** - Hardware-accelerated AES-NI, multi-threaded
- 🖥️ **Native integration** - Works with system file manager
- 🎨 **GUI available** - Easy-to-use graphical interface

## Quick Start

### GUI Mode (Windows)

Double-click to launch the GUI:
- **`crosscrypt-gui.bat`** - Command-line menu (no dependencies)
- **`crosscrypt-gui.ps1`** - PowerShell GUI (better interface)

Or run `crosscrypt.exe` without arguments to launch the default GUI.

### CLI Mode

```bash
# Windows
crosscrypt.exe --help

# macOS/Linux
./crosscrypt --help
```

### Create Encrypted Volume

```bash
# Windows
crosscrypt.exe create -d E: -l "My Secure Drive"

# macOS/Linux
./crosscrypt create -d /dev/sdb -l "My Secure Drive"
```

### Mount Encrypted Volume

```bash
crosscrypt.exe mount -d E:
```

### Unmount

```bash
crosscrypt.exe unmount -d E:
```

### Emergency Lock

```bash
crosscrypt.exe lock -d E:
```

## Download Pre-built Binaries

GitHub Actions automatically builds releases for all platforms:
- **Windows**: Download `crosscrypt-windows-x86_64.zip` from [Releases](https://github.com/yzj327850/crosscrypt/releases)
- **macOS (Intel)**: Download `crosscrypt-macos-x86_64.tar.gz`
- **macOS (Apple Silicon)**: Download `crosscrypt-macos-arm64.tar.gz`
- **Linux**: Download `crosscrypt-linux-x86_64.tar.gz`

## Building from Source

### Prerequisites

- Rust 1.70+
- Platform-specific dependencies:
  - Windows: Visual Studio Build Tools
  - macOS: Xcode Command Line Tools, macFUSE
  - Linux: libfuse3-dev

### Build

```bash
# Default build (with GUI)
cargo build --release

# CLI only (no GUI)
cargo build --release --no-default-features

# Cross-compile for Windows from Linux/macOS
cargo build --target x86_64-pc-windows-gnu --release
```

### Running

```bash
# GUI mode (default)
./crosscrypt

# CLI mode
./crosscrypt --cli
```

## Security

See [SECURITY.md](docs/SECURITY.md) for detailed security documentation.

## Architecture

See [DESIGN.md](docs/DESIGN.md) for technical design details.

## License

MIT License - See LICENSE file for details.
