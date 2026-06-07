use aes::cipher::KeyInit;
use rand::RngCore;
use std::sync::Arc;
use tracing::{debug, info};

use super::CrossCryptError;

pub const MASTER_KEY_SIZE: usize = 64; // 512 bits for XTS (2x256)
pub const SECTOR_SIZE: usize = 4096;
pub const TWEAK_SIZE: usize = 16;

pub struct CryptoEngine {
    key: Arc<[u8; MASTER_KEY_SIZE]>,
    pub sector_size: usize,
}

impl CryptoEngine {
    pub fn new(key: &[u8], sector_size: u32) -> Result<Self, CrossCryptError> {
        if key.len() != MASTER_KEY_SIZE {
            return Err(CrossCryptError::Crypto(
                format!("Invalid key size: {}, expected {}", key.len(), MASTER_KEY_SIZE)
            ));
        }
        
        let mut key_array = [0u8; MASTER_KEY_SIZE];
        key_array.copy_from_slice(key);
        
        Ok(Self {
            key: Arc::new(key_array),
            sector_size: sector_size as usize,
        })
    }
    
    /// Encrypt a single sector
    pub fn encrypt_sector(&self, sector_index: u64, data: &mut [u8]) -> Result<(), CrossCryptError> {
        if data.len() != self.sector_size {
            return Err(CrossCryptError::Crypto(
                format!("Invalid sector size: {}, expected {}", data.len(), self.sector_size)
            ));
        }
        
        // XTS mode: split key into key1 and key2
        let (key1, key2) = self.key.split_at(32);
        
        // Generate tweak from sector index
        let tweak = self.generate_tweak(sector_index, key2);
        
        // AES-256-XTS encryption
        self.xts_encrypt(key1, &tweak, data)?;
        
        Ok(())
    }
    
    /// Decrypt a single sector
    pub fn decrypt_sector(&self, sector_index: u64, data: &mut [u8]) -> Result<(), CrossCryptError> {
        if data.len() != self.sector_size {
            return Err(CrossCryptError::Crypto(
                format!("Invalid sector size: {}, expected {}", data.len(), self.sector_size)
            ));
        }
        
        let (key1, key2) = self.key.split_at(32);
        let tweak = self.generate_tweak(sector_index, key2);
        
        // AES-256-XTS decryption
        self.xts_decrypt(key1, &tweak, data)?;
        
        Ok(())
    }
    
    /// Encrypt multiple sectors in parallel
    pub fn encrypt_sectors(
        &self,
        start_sector: u64,
        data: &mut [u8],
    ) -> Result<(), CrossCryptError> {
        let num_sectors = data.len() / self.sector_size;
        
        // Use rayon for parallel processing
        use rayon::prelude::*;
        
        data.par_chunks_mut(self.sector_size)
            .enumerate()
            .try_for_each(|(i, sector)| {
                self.encrypt_sector(start_sector + i as u64, sector)
            })?;
        
        Ok(())
    }
    
    /// Decrypt multiple sectors in parallel
    pub fn decrypt_sectors(
        &self,
        start_sector: u64,
        data: &mut [u8],
    ) -> Result<(), CrossCryptError> {
        let num_sectors = data.len() / self.sector_size;
        
        use rayon::prelude::*;
        
        data.par_chunks_mut(self.sector_size)
            .enumerate()
            .try_for_each(|(i, sector)| {
                self.decrypt_sector(start_sector + i as u64, sector)
            })?;
        
        Ok(())
    }
    
    fn generate_tweak(&self, sector_index: u64, key2: &[u8]) -> [u8; 16] {
        use aes::Aes256;
        use aes::cipher::BlockEncrypt;
        
        let mut tweak = [0u8; 16];
        tweak[0..8].copy_from_slice(&sector_index.to_le_bytes());
        
        // Encrypt tweak with key2
        let cipher = Aes256::new_from_slice(key2.try_into().expect("Invalid key size")).expect("Invalid key size");
        let mut block = aes::Block::from_mut_slice(&mut tweak);
        cipher.encrypt_block(&mut block);
        
        tweak
    }
    
    fn xts_encrypt(
        &self,
        key1: &[u8],
        tweak: &[u8; 16],
        data: &mut [u8],
    ) -> Result<(), CrossCryptError> {
        use aes::Aes256;
        use aes::cipher::BlockEncrypt;
        
        let cipher = Aes256::new_from_slice(key1.try_into()
            .map_err(|_| CrossCryptError::Crypto("Invalid key size".to_string()))?)
            .map_err(|_| CrossCryptError::Crypto("Invalid key size".to_string()))?;
        
        let mut current_tweak = *tweak;
        
        for chunk in data.chunks_mut(16) {
            // XOR with tweak
            for (i, byte) in chunk.iter_mut().enumerate() {
                *byte ^= current_tweak[i];
            }
            
            // AES encrypt
            let mut block = aes::Block::from_mut_slice(chunk);
            cipher.encrypt_block(&mut block);
            
            // XOR with tweak
            for (i, byte) in chunk.iter_mut().enumerate() {
                *byte ^= current_tweak[i];
            }
            
            // Multiply tweak by x (Galois field)
            self.multiply_tweak(&mut current_tweak);
        }
        
        Ok(())
    }
    
    fn xts_decrypt(
        &self,
        key1: &[u8],
        tweak: &[u8; 16],
        data: &mut [u8],
    ) -> Result<(), CrossCryptError> {
        use aes::Aes256;
        use aes::cipher::BlockDecrypt;
        
        let cipher = Aes256::new_from_slice(key1.try_into()
            .map_err(|_| CrossCryptError::Crypto("Invalid key size".to_string()))?)
            .map_err(|_| CrossCryptError::Crypto("Invalid key size".to_string()))?;
        
        let mut current_tweak = *tweak;
        
        for chunk in data.chunks_mut(16) {
            // XOR with tweak
            for (i, byte) in chunk.iter_mut().enumerate() {
                *byte ^= current_tweak[i];
            }
            
            // AES decrypt
            let mut block = aes::Block::from_mut_slice(chunk);
            cipher.decrypt_block(&mut block);
            
            // XOR with tweak
            for (i, byte) in chunk.iter_mut().enumerate() {
                *byte ^= current_tweak[i];
            }
            
            // Multiply tweak by x (Galois field)
            self.multiply_tweak(&mut current_tweak);
        }
        
        Ok(())
    }
    
    fn multiply_tweak(&self, tweak: &mut [u8; 16]) {
        let mut carry = 0u8;
        for byte in tweak.iter_mut() {
            let new_carry = *byte >> 7;
            *byte = (*byte << 1) | carry;
            carry = new_carry;
        }
        if carry != 0 {
            tweak[0] ^= 0x87; // Reduction polynomial
        }
    }
}

/// Generate a cryptographically secure master key
pub fn generate_master_key() -> [u8; MASTER_KEY_SIZE] {
    let mut key = [0u8; MASTER_KEY_SIZE];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Generate a random salt
pub fn generate_salt(size: usize) -> Vec<u8> {
    let mut salt = vec![0u8; size];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

pub async fn run_benchmark() -> anyhow::Result<()> {
    info!("Running encryption benchmark...");
    
    let key = generate_master_key();
    let engine = CryptoEngine::new(&key, SECTOR_SIZE as u32)?;
    
    // Test different buffer sizes
    let sizes = vec![
        ("1 MB", 1 * 1024 * 1024),
        ("10 MB", 10 * 1024 * 1024),
        ("100 MB", 100 * 1024 * 1024),
        ("1 GB", 1024 * 1024 * 1024),
    ];
    
    for (name, size) in sizes {
        let mut data = vec![0u8; size];
        rand::thread_rng().fill_bytes(&mut data);
        
        let start = std::time::Instant::now();
        engine.encrypt_sectors(0, &mut data)?;
        let elapsed = start.elapsed();
        
        let throughput = size as f64 / elapsed.as_secs_f64() / (1024.0 * 1024.0);
        info!("{}: {:.2} MB/s", name, throughput);
        println!("{}: {:.2} MB/s", name, throughput);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_master_key();
        let engine = CryptoEngine::new(&key, 4096).unwrap();
        
        let original = vec![0xabu8; 4096];
        let mut encrypted = original.clone();
        
        engine.encrypt_sector(0, &mut encrypted).unwrap();
        assert_ne!(original, encrypted);
        
        engine.decrypt_sector(0, &mut encrypted).unwrap();
        assert_eq!(original, encrypted);
    }
    
    #[test]
    fn test_sector_independence() {
        let key = generate_master_key();
        let engine = CryptoEngine::new(&key, 4096).unwrap();
        
        let mut data1 = vec![0xabu8; 4096];
        let mut data2 = vec![0xabu8; 4096];
        
        engine.encrypt_sector(0, &mut data1).unwrap();
        engine.encrypt_sector(1, &mut data2).unwrap();
        
        assert_ne!(data1, data2); // Different sectors should produce different ciphertext
    }
}
