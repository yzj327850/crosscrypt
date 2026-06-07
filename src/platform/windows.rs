use std::path::Path;
use tracing::{info, warn};

use crate::core::{crypto::CryptoEngine, CrossCryptError};

pub async fn platform_mount(
    device: &Path,
    crypto: &CryptoEngine,
    mountpoint: Option<String>,
) -> Result<(), CrossCryptError> {
    info!("Mounting volume on Windows: {}", device.display());

    let drive = mountpoint.unwrap_or_else(|| {
        find_available_drive().unwrap_or_else(|| "Z:".to_string())
    });

    info!("Mounted as drive {}", drive);
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

fn find_available_drive() -> Option<String> {
    for c in b'D'..=b'Z' {
        let drive = format!("{}:", c as char);
        let path = std::path::Path::new(&drive);
        if !path.exists() {
            return Some(drive);
        }
    }
    None
}

/// WinFsp file system implementation
pub struct WinFspFilesystem {
    // TODO: Implement WinFsp callbacks
}

impl WinFspFilesystem {
    pub fn new() -> Self {
        Self {}
    }
}
