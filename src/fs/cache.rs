use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, trace};

use crate::core::CrossCryptError;
use super::block::BlockManager;

/// Cached block entry
struct CacheEntry {
    data: Vec<u8>,
    last_access: Instant,
    dirty: bool,
}

/// LRU block cache
pub struct BlockCache {
    manager: Arc<BlockManager>,
    cache: RwLock<HashMap<u64, CacheEntry>>,
    max_size: usize,
    ttl: Duration,
}

impl BlockCache {
    pub fn new(manager: Arc<BlockManager>, max_size: usize) -> Self {
        Self {
            manager,
            cache: RwLock::new(HashMap::with_capacity(max_size)),
            max_size,
            ttl: Duration::from_secs(30),
        }
    }
    
    /// Read a block (from cache or disk)
    pub async fn read(&self, block_num: u64) -> Result<Vec<u8>, CrossCryptError> {
        // Try cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&block_num) {
                trace!("Cache hit for block {}", block_num);
                return Ok(entry.data.clone());
            }
        }
        
        // Cache miss - read from disk
        trace!("Cache miss for block {}", block_num);
        let data = self.manager.read_block(block_num).await?;
        
        // Insert into cache
        self.insert(block_num, data.clone(), false).await;
        
        Ok(data)
    }
    
    /// Write a block (to cache, flush later)
    pub async fn write(&self, block_num: u64, data: Vec<u8>) -> Result<(), CrossCryptError> {
        self.insert(block_num, data, true).await;
        Ok(())
    }
    
    /// Flush dirty blocks to disk
    pub async fn flush(&self) -> Result<(), CrossCryptError> {
        let mut cache = self.cache.write().await;
        
        for (block_num, entry) in cache.iter_mut() {
            if entry.dirty {
                self.manager.write_block(*block_num, &entry.data).await?;
                entry.dirty = false;
            }
        }
        
        debug!("Cache flushed");
        Ok(())
    }
    
    /// Flush a specific block
    pub async fn flush_block(&self, block_num: u64) -> Result<(), CrossCryptError> {
        let mut cache = self.cache.write().await;
        
        if let Some(entry) = cache.get_mut(&block_num) {
            if entry.dirty {
                self.manager.write_block(block_num, &entry.data).await?;
                entry.dirty = false;
            }
        }
        
        Ok(())
    }
    
    /// Clear cache
    pub async fn clear(&self) -> Result<(), CrossCryptError> {
        self.flush().await?;
        
        let mut cache = self.cache.write().await;
        cache.clear();
        
        debug!("Cache cleared");
        Ok(())
    }
    
    async fn insert(&self, block_num: u64, data: Vec<u8>, dirty: bool) {
        let mut cache = self.cache.write().await;
        
        // Evict if necessary
        if cache.len() >= self.max_size && !cache.contains_key(&block_num) {
            self.evict_lru(&mut cache).await;
        }
        
        cache.insert(block_num, CacheEntry {
            data,
            last_access: Instant::now(),
            dirty,
        });
    }
    
    async fn evict_lru(&self, cache: &mut HashMap<u64, CacheEntry>) {
        // Find oldest entry
        let oldest = cache.iter()
            .min_by_key(|(_, entry)| entry.last_access)
            .map(|(k, _)| *k);
        
        if let Some(block_num) = oldest {
            // Flush if dirty
            if let Some(entry) = cache.get(&block_num) {
                if entry.dirty {
                    // Best effort flush
                    let _ = self.manager.write_block(block_num, &entry.data).await;
                }
            }
            
            cache.remove(&block_num);
            trace!("Evicted block {} from cache", block_num);
        }
    }
    
    /// Clean expired entries
    pub async fn clean_expired(&self) -> Result<(), CrossCryptError> {
        let now = Instant::now();
        let mut cache = self.cache.write().await;
        
        let expired: Vec<u64> = cache.iter()
            .filter(|(_, entry)| now.duration_since(entry.last_access) > self.ttl)
            .map(|(k, _)| *k)
            .collect();
        
        for block_num in expired {
            if let Some(entry) = cache.get(&block_num) {
                if entry.dirty {
                    self.manager.write_block(block_num, &entry.data).await?;
                }
            }
            cache.remove(&block_num);
        }
        
        Ok(())
    }
}
