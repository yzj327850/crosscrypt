//! Cross-platform GUI using Tauri
//!
//! This module provides a unified user interface that works across
//! Windows, macOS, and Linux.

#[cfg(feature = "gui")]
use tauri::{Manager, State};
#[cfg(feature = "gui")]
use std::sync::Arc;
#[cfg(feature = "gui")]
use tokio::sync::Mutex;

#[cfg(feature = "gui")]
use crate::core::{CrossCryptVolume, EncryptionConfig, VolumeStatus};

/// Application state
#[cfg(feature = "gui")]
pub struct AppState {
    volumes: Arc<Mutex<Vec<VolumeInfo>>>,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone, serde::Serialize)]
pub struct VolumeInfo {
    pub device: String,
    pub label: Option<String>,
    pub status: String,
    pub mounted: bool,
    pub mountpoint: Option<String>,
    pub size: u64,
    pub used: u64,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateVolumeRequest {
    pub device: String,
    pub password: String,
    pub label: Option<String>,
    pub quick: bool,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MountRequest {
    pub device: String,
    pub password: String,
    pub mountpoint: Option<String>,
}

/// Initialize the Tauri application
#[cfg(feature = "gui")]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            volumes: Arc::new(Mutex::new(vec![])),
        })
        .invoke_handler(tauri::generate_handler![
            list_volumes,
            create_volume,
            mount_volume,
            unmount_volume,
            lock_volume,
            get_volume_status,
            benchmark,
        ])
        .run(tauri::generate_context!())
        .expect("Failed to run Tauri application");
}

#[cfg(feature = "gui")]
#[tauri::command]
async fn list_volumes(state: State<'_, AppState>) -> Result<Vec<VolumeInfo>, String> {
    let volumes = state.volumes.lock().await;
    Ok(volumes.clone())
}

#[cfg(feature = "gui")]
#[tauri::command]
async fn create_volume(
    request: CreateVolumeRequest,
) -> Result<VolumeInfo, String> {
    let config = EncryptionConfig {
        algorithm: crate::core::CryptoAlgorithm::Aes256Xts,
        kdf: crate::core::KdfAlgorithm::Argon2id {
            iterations: 3,
            memory_kb: 64 * 1024,
            parallelism: 4,
        },
        sector_size: 4096,
        label: request.label.clone(),
    };

    let mut volume = CrossCryptVolume::new(request.device.clone());

    volume.create(&request.password, config, request.quick)
        .await
        .map_err(|e| e.to_string())?;

    Ok(VolumeInfo {
        device: request.device,
        label: request.label,
        status: "encrypted".to_string(),
        mounted: false,
        mountpoint: None,
        size: 0,
        used: 0,
    })
}

#[cfg(feature = "gui")]
#[tauri::command]
async fn mount_volume(
    request: MountRequest,
) -> Result<VolumeInfo, String> {
    let mut volume = CrossCryptVolume::new(request.device.clone());

    volume.mount(&request.password, request.mountpoint.clone())
        .await
        .map_err(|e| e.to_string())?;

    Ok(VolumeInfo {
        device: request.device,
        label: None,
        status: "mounted".to_string(),
        mounted: true,
        mountpoint: request.mountpoint,
        size: 0,
        used: 0,
    })
}

#[cfg(feature = "gui")]
#[tauri::command]
async fn unmount_volume(device: String) -> Result<(), String> {
    CrossCryptVolume::unmount(&device, false)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(feature = "gui")]
#[tauri::command]
async fn lock_volume(device: String) -> Result<(), String> {
    CrossCryptVolume::emergency_lock(&device)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(feature = "gui")]
#[tauri::command]
async fn get_volume_status(device: String) -> Result<String, String> {
    let volume = CrossCryptVolume::new(device);
    let status = volume.status().await.map_err(|e| e.to_string())?;

    match status {
        VolumeStatus::Encrypted => Ok("encrypted".to_string()),
        VolumeStatus::NotEncrypted => Ok("not_encrypted".to_string()),
        VolumeStatus::EncryptionInProgress => Ok("in_progress".to_string()),
    }
}

#[cfg(feature = "gui")]
#[tauri::command]
async fn benchmark() -> Result<f64, String> {
    crate::core::benchmark().await.map_err(|e| e.to_string())?;
    Ok(0.0) // TODO: Return actual benchmark results
}
