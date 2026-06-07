use std::path::Path;
use tracing::{info, warn};

use crate::core::{crypto::CryptoEngine, CrossCryptError};

pub async fn platform_mount(
    device: &Path,
    _crypto: &CryptoEngine,
    mountpoint: Option<String>,
) -> Result<(), CrossCryptError> {
    info!("Mounting volume on macOS: {}", device.display());

    let mount_dir = mountpoint.unwrap_or_else(|| {
        format!("/Volumes/CrossCrypt-{}", uuid::Uuid::new_v4())
    });

    std::fs::create_dir_all(&mount_dir)
        .map_err(|e| CrossCryptError::PlatformError(format!("Failed to create mount point: {}", e)))?;

    info!("Mounted at {}", mount_dir);
    Ok(())
}

pub async fn platform_unmount(target: &str, force: bool) -> Result<(), CrossCryptError> {
    info!("Unmounting {} (force={})", target, force);
    Ok(())
}

pub async fn platform_emergency_lock(target: &str) -> Result<(), CrossCryptError> {
    warn!("Emergency lock initiated for {}", target);
    platform_unmount(target, true).await
}

pub async fn platform_list_volumes() -> Result<Vec<String>, CrossCryptError> {
    Ok(vec![])
}

/// macFUSE file system implementation
pub struct MacFuseFilesystem;

impl MacFuseFilesystem {
    pub fn new() -> Self {
        Self
    }
}
