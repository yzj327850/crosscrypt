# CrossCrypt Design Document

## Overview

CrossCrypt is a cross-platform portable disk encryption solution designed to:
- Run directly from a mobile hard drive without installation
- Support Windows, macOS, and Linux
- Encrypt disks in-place while preserving existing data
- Provide transparent file system access after decryption
- Protect against brute force attacks with automatic lock and wipe

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                        User Interface                         │
│              (Tauri GUI + CLI + System Tray)                 │
├─────────────────────────────────────────────────────────────┤
│                      Volume Manager                           │
│         (Mount/Unmount/Lock/Status Management)               │
├─────────────────────────────────────────────────────────────┤
│                    Encryption Engine                          │
│              (AES-256-XTS + Argon2id KDF)                    │
├─────────────────────────────────────────────────────────────┤
│                    File System Layer                          │
│         (Block Manager + Cache + NTFS Parser)               │
├─────────────────────────────────────────────────────────────┤
│                   Platform Abstraction                        │
│         (WinFsp / macFUSE / FUSE3)                          │
├─────────────────────────────────────────────────────────────┤
│                     Raw Device I/O                            │
│              (Direct block device access)                    │
└─────────────────────────────────────────────────────────────┘
```

### Disk Layout

```
┌──────────────┬──────────────┬──────────────────────────────┐
│   Header     │  Superblock  │      Encrypted Data          │
│   (1 MB)     │   (16 MB)    │    (Remaining Space)         │
├──────────────┼──────────────┼──────────────────────────────┤
│              │              │                              │
│ • Magic      │ • Checksum   │ • Encrypted file system      │
│ • Version    │ • Checkpoint │ • Transparent access         │
│ • Config     │ • Journal    │ • Original data preserved    │
│ • Salt       │ • Metadata   │                              │
│ • Master Key │              │                              │
│ • Attempts   │              │                              │
│ • Lock Info  │              │                              │
│              │              │                              │
└──────────────┴──────────────┴──────────────────────────────┘
```

## Security Design

### Encryption

- **Algorithm**: AES-256-XTS
  - XTS mode provides independent encryption per sector
  - Prevents pattern leakage and targeted modification attacks
  - Suitable for disk encryption (IEEE P1619 standard)

- **Key Size**: 512 bits (two independent 256-bit keys for XTS)

- **Sector Size**: 4096 bytes (aligned with modern storage)

### Key Derivation

- **Algorithm**: Argon2id
  - Winner of Password Hashing Competition
  - Memory-hard function resistant to GPU/ASIC attacks
  - Default parameters: t=3, m=64MB, p=4

- **Salt**: 256-bit random per volume

### Brute Force Protection

```
Password Attempts:
├── 1-2 attempts: Normal operation
├── 3 attempts: Lock for 5 minutes
├── 4-9 attempts: Extended lock, exponential backoff
└── 10 attempts: Secure wipe triggered

Wipe Process:
1. Overwrite master key with random data
2. Clear all key slots
3. Overwrite critical metadata
4. Force unmount
```

### In-Place Encryption Safety

1. **Atomic Operations**: Critical updates use write-then-rename
2. **Checkpointing**: Progress saved every 1000 sectors
3. **Recovery**: Resume from last checkpoint after interruption
4. **Verification**: Full hash check before finalizing

## Platform Implementation

### Windows (WinFsp)

- User-space file system driver
- No kernel driver installation required
- Mount as drive letter or directory
- Full Windows Explorer integration

### macOS (macFUSE)

- macOS FUSE implementation
- Mount in /Volumes or custom location
- Finder integration
- Spotlight indexing support (optional)

### Linux (FUSE3)

- Native FUSE3 support
- Mount anywhere in filesystem
- Nautilus/Dolphin integration
- Systemd automount support

## Performance Optimizations

1. **SIMD Acceleration**: AES-NI for hardware-accelerated encryption
2. **Multi-threading**: Parallel sector processing with Rayon
3. **Caching**: LRU block cache with configurable size
4. **Async I/O**: Tokio for non-blocking operations
5. **Memory Mapping**: Direct mmap for large transfers

## Build System

### Cross-Compilation

```bash
# Windows
 cargo build --target x86_64-pc-windows-msvc

# macOS
 cargo build --target x86_64-apple-darwin
 cargo build --target aarch64-apple-darwin

# Linux
 cargo build --target x86_64-unknown-linux-gnu
```

### Portable Distribution

```
CrossCrypt/
├── crosscrypt.exe          # Windows executable
├── crosscrypt              # macOS/Linux executable
├── lib/
│   ├── winfsp.dll          # Windows dependencies
│   └── ...
├── portable/
│   ├── autorun.inf         # Windows auto-run
│   ├── CrossCrypt.app      # macOS app bundle
│   └── crosscrypt.desktop  # Linux desktop entry
└── README.md
```

## Testing Strategy

1. **Unit Tests**: Core crypto, format, KDF
2. **Integration Tests**: Full encrypt/decrypt cycles
3. **Platform Tests**: OS-specific mount/unmount
4. **Fuzz Tests**: Random input handling
5. **Performance Tests**: Benchmark suite

## Future Enhancements

- [ ] Hardware security key support (YubiKey)
- [ ] Hidden volumes (plausible deniability)
- [ ] Network share encryption
- [ ] Cloud storage integration
- [ ] Mobile app companion
