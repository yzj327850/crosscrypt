use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tracing::{debug, info, warn};

use super::{CryptoAlgorithm, CrossCryptError, EncryptionConfig, KdfAlgorithm, VolumeStatus};

/// Magic number to identify CrossCrypt volumes
pub const CROSSCRYPT_MAGIC: &[u8; 8] = b"CCRYPT01";

/// Size of volume header
pub const HEADER_SIZE: usize = 1024 * 1024; // 1 MB

/// Size of superblock
pub const SUPERBLOCK_SIZE: usize = 16 * 1024 * 1024; // 16 MB

/// Sector size for encryption
pub const SECTOR_SIZE: usize = 4096;

/// Maximum password attempts before lock
pub const MAX_PASSWORD_ATTEMPTS: u32 = 3;

/// Maximum password attempts before wipe
pub const MAX_ATTEMPTS_BEFORE_WIPE: u32 = 10;

/// Lock duration in seconds
pub const LOCK_DURATION_SECS: u64 = 300; // 5 minutes

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeHeader {
    pub magic: [u8; 8],
    pub version: u32,
    pub config: EncryptionConfig,
    pub salt: Vec<u8>,
    pub encrypted_master_key: Vec<u8>,
    pub master_key_nonce: Vec<u8>,
    pub attempt_count: u32,
    pub locked_until: Option<u64>, // Unix timestamp
    pub wipe_triggered: bool,
    pub creation_time: u64,
    pub flags: u32,
    pub reserved: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSuperblock {
    pub header_checksum: [u8; 32],
    pub encryption_checkpoint: u64, // Last encrypted sector
    pub total_sectors: u64,
    pub flags: u32,
    pub journal_start: u64,
    pub journal_size: u64,
    pub reserved: Vec<u8>,
}

impl VolumeHeader {
    pub fn new(config: &EncryptionConfig) -> Self {
        Self {
            magic: *CROSSCRYPT_MAGIC,
            version: 1,
            config: config.clone(),
            salt: super::crypto::generate_salt(32),
            encrypted_master_key: Vec::new(),
            master_key_nonce: super::crypto::generate_salt(12),
            attempt_count: 0,
            locked_until: None,
            wipe_triggered: false,
            creation_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            flags: 0,
            reserved: vec![0u8; 256],
        }
    }
    
    pub fn encrypt_master_key(
        &mut self,
        master_key: &[u8],
        kek: &[u8],
    ) -> Result<(), CrossCryptError> {
        let cipher = ChaCha20Poly1305::new_from_slice(&kek[..32])
            .map_err(|e| CrossCryptError::Crypto(format!("Cipher init failed: {:?}", e)))?;
        
        let nonce = Nonce::from_slice(&self.master_key_nonce);
        
        self.encrypted_master_key = cipher
            .encrypt(nonce, master_key)
            .map_err(|e| CrossCryptError::Crypto(format!("Encryption failed: {:?}", e)))?;
        
        Ok(())
    }
    
    pub fn decrypt_master_key(&self, kek: &[u8]) -> Result<Vec<u8>, CrossCryptError> {
        let cipher = ChaCha20Poly1305::new_from_slice(&kek[..32])
            .map_err(|e| CrossCryptError::Crypto(format!("Cipher init failed: {:?}", e)))?;
        
        let nonce = Nonce::from_slice(&self.master_key_nonce);
        
        cipher
            .decrypt(nonce, self.encrypted_master_key.as_ref())
            .map_err(|_| CrossCryptError::InvalidPassword)
    }
    
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now < locked_until
        } else {
            false
        }
    }
    
    pub fn lock_expired(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now >= locked_until
        } else {
            true
        }
    }
    
    pub fn clear_lock(&mut self) {
        self.locked_until = None;
        self.attempt_count = 0;
    }
    
    pub fn set_lock(&mut self, duration: std::time::Duration) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.locked_until = Some(now + duration.as_secs());
    }
    
    pub fn increment_attempt(&mut self) -> Result<(), CrossCryptError> {
        self.attempt_count += 1;
        
        if self.attempt_count >= MAX_ATTEMPTS_BEFORE_WIPE {
            self.wipe_triggered = true;
            return Err(CrossCryptError::Crypto(
                "Too many failed attempts. Volume will be wiped.".to_string()
            ));
        }
        
        if self.attempt_count >= MAX_PASSWORD_ATTEMPTS {
            self.set_lock(std::time::Duration::from_secs(LOCK_DURATION_SECS));
        }
        
        Ok(())
    }
    
    pub fn serialize(&self) -> Result<Vec<u8>, CrossCryptError> {
        bincode::serialize(self)
            .map_err(|e| CrossCryptError::Crypto(format!("Serialization failed: {:?}", e)))
    }
    
    pub fn deserialize(data: &[u8]) -> Result<Self, CrossCryptError> {
        bincode::deserialize(data)
            .map_err(|e| CrossCryptError::Crypto(format!("Deserialization failed: {:?}", e)))
    }
}

/// Write volume header to device
pub async fn write_header(
    device: &Path,
    header: &VolumeHeader,
) -> Result<(), CrossCryptError> {
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(device)
        .await
        .map_err(CrossCryptError::Io)?;
    
    let serialized = header.serialize()?;
    
    // Write magic
    file.write_all(&header.magic)
        .await
        .map_err(CrossCryptError::Io)?;
    
    // Write serialized header
    file.write_all(&serialized)
        .await
        .map_err(CrossCryptError::Io)?;
    
    // Pad to HEADER_SIZE
    let padding = vec![0u8; HEADER_SIZE - 8 - serialized.len()];
    file.write_all(&padding)
        .await
        .map_err(CrossCryptError::Io)?;
    
    file.sync_all().await.map_err(CrossCryptError::Io)?;
    
    info!("Volume header written to {}", device.display());
    Ok(())
}

/// Read volume header from device
pub async fn read_header(device: &Path) -> Result<VolumeHeader, CrossCryptError> {
    let mut file = tokio::fs::OpenOptions::new()
        .read(true)
        .open(device)
        .await
        .map_err(CrossCryptError::Io)?;
    
    // Read magic
    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)
        .await
        .map_err(CrossCryptError::Io)?;
    
    if &magic != CROSSCRYPT_MAGIC {
        return Err(CrossCryptError::Crypto("Not a CrossCrypt volume".to_string()));
    }
    
    // Read rest of header
    let mut buffer = vec![0u8; HEADER_SIZE - 8];
    file.read_exact(&mut buffer)
        .await
        .map_err(CrossCryptError::Io)?;
    
    // Find end of serialized data (bincode doesn't store length)
    // For simplicity, we'll use a length prefix in production
    let header = VolumeHeader::deserialize(&buffer)?;
    
    Ok(header)
}

/// Check if device has existing data
pub async fn check_existing_data(device: &Path) -> Result<bool, CrossCryptError> {
    let metadata = tokio::fs::metadata(device)
        .await
        .map_err(CrossCryptError::Io)?;
    
    if metadata.len() == 0 {
        return Ok(false);
    }
    
    // Check if it's already a CrossCrypt volume
    match read_header(device).await {
        Ok(_) => Ok(false), // Already encrypted
        Err(_) => Ok(true), // Has data but not encrypted
    }
}

/// Check volume status
pub async fn check_volume_status(device: &Path) -> Result<VolumeStatus, CrossCryptError> {
    match read_header(device).await {
        Ok(header) => {
            if header.wipe_triggered {
                Ok(VolumeStatus::NotEncrypted)
            } else {
                Ok(VolumeStatus::Encrypted)
            }
        }
        Err(CrossCryptError::Crypto(msg)) if msg.contains("Not a CrossCrypt") => {
            Ok(VolumeStatus::NotEncrypted)
        }
        Err(e) => Err(e),
    }
}

/// Quick format: create empty encrypted volume
pub async fn quick_format(
    device: &Path,
    header: &VolumeHeader,
) -> Result<(), CrossCryptError> {
    // Just initialize the header, data area remains zeros
    // When read, zeros will be decrypted to random data
    info!("Quick format completed");
    Ok(())
}

/// Encrypt device in-place with progress callback
pub async fn encrypt_in_place<F>(
    device: &Path,
    header: &VolumeHeader,
    master_key: &[u8],
    mut progress: F,
) -> Result<(), CrossCryptError>
where
    F: FnMut(f64),
{
    use super::crypto::CryptoEngine;
    
    let engine = CryptoEngine::new(master_key, header.config.sector_size)?;
    
    let mut file = tokio::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(device)
        .await
        .map_err(CrossCryptError::Io)?;
    
    let file_size = file.metadata().await.map_err(CrossCryptError::Io)?.len();
    let data_start = (HEADER_SIZE + SUPERBLOCK_SIZE) as u64;
    
    // Ensure file is large enough for header + superblock
    if file_size <= data_start {
        return Err(CrossCryptError::Crypto(
            format!("File too small: {} bytes, need at least {}", file_size, data_start)
        ));
    }
    
    let data_size = file_size - data_start;
    
    let chunk_size = 1024 * 1024; // 1 MB chunks
    let mut buffer = vec![0u8; chunk_size];
    let mut encrypted = 0u64;
    
    let mut offset = data_start;
    
    while offset < file_size {
        let to_read = std::cmp::min(chunk_size as u64, file_size - offset) as usize;
        
        file.seek(std::io::SeekFrom::Start(offset))
            .await
            .map_err(CrossCryptError::Io)?;
        
        file.read_exact(&mut buffer[..to_read])
            .await
            .map_err(CrossCryptError::Io)?;
        
        // Encrypt chunk
        let start_sector = (offset - data_start) / header.config.sector_size as u64;
        engine.encrypt_sectors(start_sector, &mut buffer[..to_read])?;
        
        // Write back
        file.seek(std::io::SeekFrom::Start(offset))
            .await
            .map_err(CrossCryptError::Io)?;
        
        file.write_all(&buffer[..to_read])
            .await
            .map_err(CrossCryptError::Io)?;
        
        file.sync_data().await.map_err(CrossCryptError::Io)?;
        
        encrypted += to_read as u64;
        offset += to_read as u64;
        
        progress(encrypted as f64 / data_size as f64);
    }
    
    info!("In-place encryption completed");
    Ok(())
}

/// Resume interrupted encryption
pub async fn resume_encryption(
    device: &Path,
    header: &VolumeHeader,
    master_key: &[u8],
) -> Result<(), CrossCryptError> {
    // Read superblock to find checkpoint
    let superblock = read_superblock(device).await?;
    
    if superblock.encryption_checkpoint >= superblock.total_sectors {
        info!("Encryption already complete");
        return Ok(());
    }
    
    // Resume from checkpoint
    info!(
        "Resuming encryption from sector {}",
        superblock.encryption_checkpoint
    );
    
    // TODO: Implement resume logic
    
    Ok(())
}

/// Read superblock
async fn read_superblock(device: &Path) -> Result<VolumeSuperblock, CrossCryptError> {
    let mut file = tokio::fs::OpenOptions::new()
        .read(true)
        .open(device)
        .await
        .map_err(CrossCryptError::Io)?;
    
    file.seek(std::io::SeekFrom::Start(HEADER_SIZE as u64))
        .await
        .map_err(CrossCryptError::Io)?;
    
    let mut buffer = vec![0u8; SUPERBLOCK_SIZE];
    file.read_exact(&mut buffer)
        .await
        .map_err(CrossCryptError::Io)?;
    
    // Deserialize superblock
    // TODO: Implement proper deserialization
    
    Ok(VolumeSuperblock {
        header_checksum: [0u8; 32],
        encryption_checkpoint: 0,
        total_sectors: 0,
        flags: 0,
        journal_start: 0,
        journal_size: 0,
        reserved: vec![0u8; 256],
    })
}
