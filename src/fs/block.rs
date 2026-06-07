use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, trace};

use crate::core::{crypto::CryptoEngine, CrossCryptError};

/// Manages block-level I/O with encryption/decryption
pub struct BlockManager {
    device: Arc<RwLock<tokio::fs::File>>,
    crypto: Arc<CryptoEngine>,
    pub sector_size: usize,
    data_offset: u64, // Offset where encrypted data starts
}

impl BlockManager {
    pub fn new(
        device: tokio::fs::File,
        crypto: CryptoEngine,
        data_offset: u64,
    ) -> Self {
        let sector_size = crypto.sector_size;
        Self {
            device: Arc::new(RwLock::new(device)),
            crypto: Arc::new(crypto),
            sector_size,
            data_offset,
        }
    }
    
    /// Read and decrypt a block
    pub async fn read_block(&self, block_num: u64) -> Result<Vec<u8>, CrossCryptError> {
        let offset = self.data_offset + block_num * self.sector_size as u64;
        
        let mut device = self.device.write().await;
        
        use tokio::io::{AsyncReadExt, AsyncSeekExt};
        device.seek(std::io::SeekFrom::Start(offset))
            .await
            .map_err(CrossCryptError::Io)?;
        
        let mut buffer = vec![0u8; self.sector_size];
        device.read_exact(&mut buffer)
            .await
            .map_err(CrossCryptError::Io)?;
        
        drop(device);
        
        // Decrypt
        self.crypto.decrypt_sector(block_num, &mut buffer)?;
        
        trace!("Read block {}", block_num);
        Ok(buffer)
    }
    
    /// Encrypt and write a block
    pub async fn write_block(
        &self,
        block_num: u64,
        data: &[u8],
    ) -> Result<(), CrossCryptError> {
        if data.len() != self.sector_size {
            return Err(CrossCryptError::Crypto(
                format!("Block size mismatch: {} != {}", data.len(), self.sector_size)
            ));
        }
        
        let mut encrypted = data.to_vec();
        self.crypto.encrypt_sector(block_num, &mut encrypted)?;
        
        let offset = self.data_offset + block_num * self.sector_size as u64;
        
        let mut device = self.device.write().await;
        
        use tokio::io::{AsyncSeekExt, AsyncWriteExt};
        device.seek(std::io::SeekFrom::Start(offset))
            .await
            .map_err(CrossCryptError::Io)?;
        
        device.write_all(&encrypted)
            .await
            .map_err(CrossCryptError::Io)?;
        
        device.sync_data().await.map_err(CrossCryptError::Io)?;
        
        trace!("Wrote block {}", block_num);
        Ok(())
    }
    
    /// Read multiple contiguous blocks
    pub async fn read_blocks(
        &self,
        start_block: u64,
        count: usize,
    ) -> Result<Vec<u8>, CrossCryptError> {
        let mut result = Vec::with_capacity(count * self.sector_size);
        
        for i in 0..count {
            let block = self.read_block(start_block + i as u64).await?;
            result.extend_from_slice(&block);
        }
        
        Ok(result)
    }
    
    /// Write multiple contiguous blocks
    pub async fn write_blocks(
        &self,
        start_block: u64,
        data: &[u8],
    ) -> Result<(), CrossCryptError> {
        let num_blocks = data.len() / self.sector_size;
        
        for i in 0..num_blocks {
            let offset = i * self.sector_size;
            self.write_block(start_block + i as u64, &data[offset..offset + self.sector_size])
                .await?;
        }
        
        Ok(())
    }
    
    /// Get total number of blocks
    pub async fn total_blocks(&self) -> Result<u64, CrossCryptError> {
        let device = self.device.read().await;
        let metadata = device.metadata().await.map_err(CrossCryptError::Io)?;
        let data_size = metadata.len() - self.data_offset;
        Ok(data_size / self.sector_size as u64)
    }
}
