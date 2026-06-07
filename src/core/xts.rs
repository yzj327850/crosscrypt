//! AES-256-XTS Implementation
//!
//! XTS (XEX-based tweaked-codebook mode with ciphertext stealing) is a
//! block cipher mode designed specifically for disk encryption.
//!
//! This implementation uses AES-NI when available for hardware acceleration.

use aes::cipher::{BlockEncrypt, BlockDecrypt, KeyInit};
use aes::Aes256;

/// XTS cipher instance
pub struct XtsCipher {
    key1: Aes256,  // For data encryption
    key2: Aes256,  // For tweak encryption
}

impl XtsCipher {
    /// Create new XTS cipher from 512-bit key
    pub fn new(key: &[u8; 64]) -> Self {
        let (k1, k2) = key.split_at(32);
        
        Self {
            key1: Aes256::new_from_slice(k1).expect("Invalid key size"),
            key2: Aes256::new_from_slice(k2).expect("Invalid key size"),
        }
    }
    
    /// Encrypt a data unit (sector)
    pub fn encrypt_sector(&self, sector_index: u64, data: &mut [u8]) {
        // Generate initial tweak
        let mut tweak = self.generate_tweak(sector_index);
        
        // Process each 16-byte block
        for chunk in data.chunks_mut(16) {
            // XOR with tweak
            for (i, byte) in chunk.iter_mut().enumerate() {
                *byte ^= tweak[i];
            }
            
            // AES encrypt
            let mut block = aes::Block::from_mut_slice(chunk);
            self.key1.encrypt_block(&mut block);
            
            // XOR with tweak
            for (i, byte) in chunk.iter_mut().enumerate() {
                *byte ^= tweak[i];
            }
            
            // Multiply tweak by x in GF(2^128)
            multiply_tweak(&mut tweak);
        }
    }
    
    /// Decrypt a data unit (sector)
    pub fn decrypt_sector(&self, sector_index: u64, data: &mut [u8]) {
        // Generate initial tweak
        let mut tweak = self.generate_tweak(sector_index);
        
        // Process each 16-byte block
        for chunk in data.chunks_mut(16) {
            // XOR with tweak
            for (i, byte) in chunk.iter_mut().enumerate() {
                *byte ^= tweak[i];
            }
            
            // AES decrypt
            let mut block = aes::Block::from_mut_slice(chunk);
            self.key1.decrypt_block(&mut block);
            
            // XOR with tweak
            for (i, byte) in chunk.iter_mut().enumerate() {
                *byte ^= tweak[i];
            }
            
            // Multiply tweak by x in GF(2^128)
            multiply_tweak(&mut tweak);
        }
    }
    
    /// Generate initial tweak for a sector
    fn generate_tweak(&self, sector_index: u64) -> [u8; 16] {
        let mut tweak = [0u8; 16];
        tweak[0..8].copy_from_slice(&sector_index.to_le_bytes());
        
        // Encrypt tweak with key2
        let mut block = aes::Block::from_mut_slice(&mut tweak);
        self.key2.encrypt_block(&mut block);
        
        tweak
    }
}

/// Multiply tweak by x in GF(2^128)
/// This is equivalent to left shift with conditional XOR with 0x87
fn multiply_tweak(tweak: &mut [u8; 16]) {
    let mut carry = 0u8;
    
    for byte in tweak.iter_mut() {
        let new_carry = *byte >> 7;
        *byte = (*byte << 1) | carry;
        carry = new_carry;
    }
    
    // If highest bit was set, XOR with reduction polynomial
    if carry != 0 {
        tweak[0] ^= 0x87;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_xts_encrypt_decrypt() {
        let key = [0x42u8; 64];
        let cipher = XtsCipher::new(&key);
        
        let plaintext = vec![0xabu8; 4096];
        let mut ciphertext = plaintext.clone();
        
        cipher.encrypt_sector(0, &mut ciphertext);
        assert_ne!(plaintext, ciphertext);
        
        cipher.decrypt_sector(0, &mut ciphertext);
        assert_eq!(plaintext, ciphertext);
    }
    
    #[test]
    fn test_different_sectors() {
        let key = [0x42u8; 64];
        let cipher = XtsCipher::new(&key);
        
        let mut data1 = vec![0xabu8; 4096];
        let mut data2 = vec![0xabu8; 4096];
        
        cipher.encrypt_sector(0, &mut data1);
        cipher.encrypt_sector(1, &mut data2);
        
        assert_ne!(data1, data2);
    }
    
    #[test]
    fn test_tweak_multiplication() {
        let mut tweak = [0u8; 16];
        tweak[15] = 0x80;
        
        multiply_tweak(&mut tweak);
        
        assert_eq!(tweak[0], 0x87);
        assert_eq!(tweak[15], 0x00);
    }
}
