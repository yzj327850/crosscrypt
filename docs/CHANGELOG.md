# Changelog

All notable changes to CrossCrypt will be documented in this file.

## [0.1.0] - 2024-01-01

### Added
- Initial release
- AES-256-XTS disk encryption
- Argon2id key derivation
- In-place encryption preserving existing data
- Cross-platform architecture (Windows/macOS/Linux)
- CLI interface for create/mount/unmount/lock operations
- Brute force protection with auto-lock and secure wipe
- NTFS boot sector parser
- MFT entry parser
- Block-level I/O with transparent encryption
- LRU block cache
- Parallel sector encryption/decryption
- Hardware AES-NI acceleration support
- Resume interrupted encryption
- Platform-specific mount abstractions
- Portable deployment scripts

### Security
- AES-256-XTS encryption (NIST standard)
- Argon2id memory-hard KDF
- Password attempt limiting (3 attempts = 5min lock, 10 attempts = wipe)
- Secure memory handling with automatic key clearing
- Constant-time password verification

## [Unreleased]

### Planned
- Full NTFS file system driver
- Write support for encrypted volumes
- GUI interface using Tauri
- Hardware security key support (YubiKey)
- Hidden volumes (plausible deniability)
- Network share encryption
- Cloud storage integration
- Mobile app companion
