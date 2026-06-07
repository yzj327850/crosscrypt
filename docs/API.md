# CrossCrypt API Documentation

## Core Module

### `CrossCryptVolume`

Main volume management struct.

```rust
pub struct CrossCryptVolume {
    // ...
}
```

#### Methods

##### `new(device: String) -> Self`

Create a new volume instance for the given device path.

##### `create(&mut self, password: &str, config: EncryptionConfig, quick: bool) -> Result<()>`

Create a new encrypted volume.

- `password` - Encryption password
- `config` - Encryption configuration
- `quick` - If true, create empty volume. If false, encrypt existing data in-place.

##### `mount(&mut self, password: &str, mountpoint: Option<String>) -> Result<()>`

Mount an encrypted volume.

- `password` - Decryption password
- `mountpoint` - Optional mount point (drive letter on Windows, path on Unix)

##### `unmount(target: &str, force: bool) -> Result<()>`

Unmount a volume.

##### `emergency_lock(target: &str) -> Result<()>`

Emergency lock - force unmount and clear keys from memory.

##### `status(&self) -> Result<VolumeStatus>`

Check volume status.

##### `resume_encryption(&mut self, password: &str) -> Result<()>`

Resume interrupted in-place encryption.

### `EncryptionConfig`

```rust
pub struct EncryptionConfig {
    pub algorithm: CryptoAlgorithm,
    pub kdf: KdfAlgorithm,
    pub sector_size: u32,
    pub label: Option<String>,
}
```

### `CryptoAlgorithm`

```rust
pub enum CryptoAlgorithm {
    Aes256Xts,
}
```

### `KdfAlgorithm`

```rust
pub enum KdfAlgorithm {
    Argon2id {
        iterations: u32,
        memory_kb: u32,
        parallelism: u32,
    },
}
```

### `VolumeStatus`

```rust
pub enum VolumeStatus {
    Encrypted,
    NotEncrypted,
    EncryptionInProgress,
}
```

## Crypto Module

### `CryptoEngine`

Low-level encryption/decryption engine.

```rust
pub struct CryptoEngine {
    pub sector_size: usize,
}
```

#### Methods

##### `new(key: &[u8], sector_size: u32) -> Result<Self>`

Create new crypto engine with 512-bit key (two 256-bit keys for XTS).

##### `encrypt_sector(&self, sector_index: u64, data: &mut [u8]) -> Result<()>`

Encrypt a single sector in-place.

##### `decrypt_sector(&self, sector_index: u64, data: &mut [u8]) -> Result<()>`

Decrypt a single sector in-place.

##### `encrypt_sectors(&self, start_sector: u64, data: &mut [u8]) -> Result<()>`

Encrypt multiple sectors in parallel.

##### `decrypt_sectors(&self, start_sector: u64, data: &mut [u8]) -> Result<()>`

Decrypt multiple sectors in parallel.

### Key Derivation

#### `KdfEngine::argon2id(password, salt, iterations, memory_kb, parallelism) -> Result<Vec<u8>>`

Derive encryption key using Argon2id.

## NTFS Module

### `NtfsFilesystem`

Read-only NTFS parser for encrypted volumes.

```rust
pub struct NtfsFilesystem {
    // ...
}
```

#### Methods

##### `new(block_manager: BlockManager) -> Self`

Create new NTFS filesystem instance.

##### `parse_boot_sector(&mut self) -> Result<()>`

Parse NTFS boot sector.

##### `read_mft_entry(&mut self, record_number: u64) -> Result<MftEntry>`

Read and parse MFT entry.

## Block Module

### `BlockManager`

Block-level I/O with transparent encryption/decryption.

```rust
pub struct BlockManager {
    pub sector_size: usize,
}
```

#### Methods

##### `new(device: File, crypto: CryptoEngine, data_offset: u64) -> Self`

Create new block manager.

##### `read_block(&self, block_num: u64) -> Result<Vec<u8>>`

Read and decrypt a block.

##### `write_block(&self, block_num: u64, data: &[u8]) -> Result<()>`

Encrypt and write a block.

## Error Types

### `CrossCryptError`

```rust
pub enum CrossCryptError {
    Io(std::io::Error),
    Crypto(String),
    InvalidPassword,
    VolumeLocked,
    EncryptionInProgress,
    DeviceNotFound(String),
    PlatformError(String),
}
```

## Examples

### Create Encrypted Volume

```rust
use crosscrypt::core::{CrossCryptVolume, EncryptionConfig, KdfAlgorithm, CryptoAlgorithm};

let config = EncryptionConfig {
    algorithm: CryptoAlgorithm::Aes256Xts,
    kdf: KdfAlgorithm::Argon2id {
        iterations: 3,
        memory_kb: 64 * 1024,
        parallelism: 4,
    },
    sector_size: 4096,
    label: Some("My Drive".to_string()),
};

let mut volume = CrossCryptVolume::new("/dev/sdb".to_string());
volume.create("my_password", config, false).await?;
```

### Mount Volume

```rust
let mut volume = CrossCryptVolume::new("/dev/sdb".to_string());
volume.mount("my_password", Some("/mnt/mydrive".to_string())).await?;
```

### Read NTFS Data

```rust
use crosscrypt::fs::{BlockManager, NtfsFilesystem};

let block_manager = BlockManager::new(file, crypto, data_offset);
let mut ntfs = NtfsFilesystem::new(block_manager);
ntfs.parse_boot_sector().await?;

let entry = ntfs.read_mft_entry(5).await?; // Root directory
```
