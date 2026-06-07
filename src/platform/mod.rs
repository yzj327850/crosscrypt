use std::path::Path;

use crate::core::{crypto::CryptoEngine, CrossCryptError};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
pub use windows::*;
#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(target_os = "linux")]
pub use linux::*;

/// Platform-specific volume mount
pub async fn mount_volume(
    device: &Path,
    crypto: &CryptoEngine,
    mountpoint: Option<String>,
) -> Result<(), CrossCryptError> {
    platform_mount(device, crypto, mountpoint).await
}

/// Platform-specific volume unmount
pub async fn unmount_volume(target: &str, force: bool) -> Result<(), CrossCryptError> {
    platform_unmount(target, force).await
}

/// Emergency lock
pub async fn emergency_lock(target: &str) -> Result<(), CrossCryptError> {
    platform_emergency_lock(target).await
}

/// List CrossCrypt volumes
pub async fn list_volumes() -> Result<Vec<String>, CrossCryptError> {
    platform_list_volumes().await
}
