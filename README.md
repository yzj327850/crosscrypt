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

## Quick Start

### Installation

Download the latest release for your platform and extract to your USB drive.

### Encrypt a New Drive

```bash
# Windows
crosscrypt.exe create -d E: -l "My Secure Drive"

# macOS/Linux
./crosscrypt create -d /dev/sdb -l "My Secure Drive"
```

### Mount an Encrypted Drive

```bash
crosscrypt mount -d E:
```

### Unmount

```bash
crosscrypt unmount -d E:
```

### Emergency Lock

```bash
crosscrypt lock -d E:
```

## Building from Source

### Prerequisites

- Rust 1.70+
- Platform-specific dependencies:
  - Windows: Visual Studio Build Tools
  - macOS: Xcode Command Line Tools, macFUSE
  - Linux: libfuse3-dev

### Build

```bash
cargo build --release
```

### Cross-compile

```bash
# Windows from Linux/macOS
cargo build --target x86_64-pc-windows-gnu --release

# macOS from Linux
cargo build --target x86_64-apple-darwin --release
```

## Security

See [SECURITY.md](docs/SECURITY.md) for detailed security documentation.

## Architecture

See [DESIGN.md](docs/DESIGN.md) for technical design details.

## License

MIT License - See LICENSE file for details.
