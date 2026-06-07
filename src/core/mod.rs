pub mod crypto;
pub mod format;
pub mod kdf;
pub mod xts;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

pub use crypto::CryptoEngine;
pub use format::{VolumeHeader, VolumeSuperblock};
pub use kdf::{Argon2idParams, KdfEngine};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CryptoAlgorithm {
    Aes256Xts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KdfAlgorithm {
    Argon2id {
        iterations: u32,
        memory_kb: u32,
        parallelism: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: CryptoAlgorithm,
    pub kdf: KdfAlgorithm,
    pub sector_size: u32,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VolumeStatus {
    Encrypted,
    NotEncrypted,
    EncryptionInProgress,
}

#[derive(Error, Debug)]
pub enum CrossCryptError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Volume locked")]
    VolumeLocked,

    #[error("Encryption in progress")]
    EncryptionInProgress,

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Platform error: {0}")]
    PlatformError(String),
}

pub struct CrossCryptVolume {
    device: PathBuf,
    header: Option<VolumeHeader>,
    crypto: Option<CryptoEngine>,
}

impl CrossCryptVolume {
    pub fn new(device: String) -> Self {
        Self {
            device: PathBuf::from(device),
            header: None,
            crypto: None,
        }
    }

    pub async fn check_existing_data(&mut self) -> anyhow::Result<bool> {
        format::check_existing_data(&self.device)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn create(
        &mut self,
        password: &str,
        config: EncryptionConfig,
        quick: bool,
    ) -> anyhow::Result<()> {
        let mut header = VolumeHeader::new(&config);

        // Generate master key
        let master_key = crypto::generate_master_key();

        // Derive key encryption key from password
        let kek = match &config.kdf {
            KdfAlgorithm::Argon2id { iterations, memory_kb, parallelism } => {
                KdfEngine::argon2id(
                    password.as_bytes(),
                    &header.salt,
                    *iterations,
                    *memory_kb,
                    *parallelism,
                )?
            }
        };

        // Encrypt master key with KEK
        header.encrypt_master_key(&master_key, &kek)?;

        // Write header to device
        format::write_header(&self.device, &header).await?;

        if quick {
            // Quick format: just initialize empty encrypted volume
            format::quick_format(&self.device, &header).await?;
        } else {
            // Full in-place encryption
            self.encrypt_in_place(&header, &master_key).await?;
        }

        self.header = Some(header);
        Ok(())
    }

    async fn encrypt_in_place(
        &mut self,
        header: &VolumeHeader,
        master_key: &[u8],
    ) -> anyhow::Result<()> {
        format::encrypt_in_place(
            &self.device,
            header,
            master_key,
            |progress| {
                print!("\rEncrypting: {:.1}%", progress * 100.0);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        ).await?;

        println!(); // New line after progress
        Ok(())
    }

    pub async fn mount(
        &mut self,
        password: &str,
        mountpoint: Option<String>,
    ) -> anyhow::Result<()> {
        // Read header
        let mut header = format::read_header(&self.device).await?;

        // Check if volume is locked
        if header.is_locked() {
            if header.lock_expired() {
                header.clear_lock();
            } else {
                anyhow::bail!(CrossCryptError::VolumeLocked);
            }
        }

        // Derive KEK
        let kek = match &header.config.kdf {
            KdfAlgorithm::Argon2id { iterations, memory_kb, parallelism } => {
                KdfEngine::argon2id(
                    password.as_bytes(),
                    &header.salt,
                    *iterations,
                    *memory_kb,
                    *parallelism,
                )?
            }
        };

        // Decrypt master key
        let master_key = header.decrypt_master_key(&kek)
            .map_err(|_| CrossCryptError::InvalidPassword)?;

        // Initialize crypto engine
        let crypto = CryptoEngine::new(&master_key, header.config.sector_size)?;

        // Platform-specific mount
        crate::platform::mount_volume(&self.device, &crypto, mountpoint)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        self.header = Some(header);
        self.crypto = Some(crypto);

        Ok(())
    }

    pub async fn unmount(target: &str, force: bool) -> anyhow::Result<()> {
        crate::platform::unmount_volume(target, force)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn emergency_lock(target: &str) -> anyhow::Result<()> {
        crate::platform::emergency_lock(target)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn status(&self) -> anyhow::Result<VolumeStatus> {
        format::check_volume_status(&self.device)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn resume_encryption(&mut self, password: &str) -> anyhow::Result<()> {
        let header = format::read_header(&self.device).await?;

        // Derive KEK
        let kek = match &header.config.kdf {
            KdfAlgorithm::Argon2id { iterations, memory_kb, parallelism } => {
                KdfEngine::argon2id(
                    password.as_bytes(),
                    &header.salt,
                    *iterations,
                    *memory_kb,
                    *parallelism,
                )?
            }
        };

        let master_key = header.decrypt_master_key(&kek)
            .map_err(|_| CrossCryptError::InvalidPassword)?;

        // Resume from last checkpoint
        format::resume_encryption(&self.device, &header, &master_key).await?;

        Ok(())
    }

    pub async fn list_volumes() -> anyhow::Result<Vec<String>> {
        crate::platform::list_volumes()
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn lock(&mut self) -> anyhow::Result<()> {
        if let Some(header) = &mut self.header {
            header.set_lock(std::time::Duration::from_secs(300)); // 5 minutes
            format::write_header(&self.device, header).await?;
        }
        Ok(())
    }
}

pub async fn benchmark() -> anyhow::Result<()> {
    crypto::run_benchmark().await
}
