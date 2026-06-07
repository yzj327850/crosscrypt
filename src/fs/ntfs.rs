//! NTFS Filesystem Parser
//!
//! This module provides read-only NTFS parsing on top of encrypted blocks.
//! It allows transparent access to existing NTFS data after decryption.

use std::collections::HashMap;
use std::ffi::OsString;
use std::time::SystemTime;
use tracing::{debug, trace, warn};

use crate::core::CrossCryptError;
use super::block::BlockManager;
use super::encrypt_fs::{EncryptFileSystem, FileAttr, FileType, Statfs};

/// NTFS Boot Sector
#[derive(Debug, Clone)]
pub struct NtfsBootSector {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub total_sectors: u64,
    pub mft_cluster: u64,
    pub mft_mirror_cluster: u64,
    pub clusters_per_mft_record: i8,
    pub clusters_per_index_record: i8,
    pub serial_number: u64,
}

/// NTFS MFT Entry
#[derive(Debug, Clone)]
pub struct MftEntry {
    pub signature: [u8; 4],
    pub flags: u16,
    pub used_size: u32,
    pub allocated_size: u32,
    pub base_record: u64,
    pub next_attribute_id: u16,
    pub attributes: Vec<MftAttribute>,
}

/// NTFS Attribute Types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttributeType {
    StandardInformation = 0x10,
    AttributeList = 0x20,
    FileName = 0x30,
    ObjectId = 0x40,
    SecurityDescriptor = 0x50,
    VolumeName = 0x60,
    VolumeInformation = 0x70,
    Data = 0x80,
    IndexRoot = 0x90,
    IndexAllocation = 0xA0,
    Bitmap = 0xB0,
    ReparsePoint = 0xC0,
    EaInformation = 0xD0,
    Ea = 0xE0,
    LoggedUtilityStream = 0x100,
    End = 0xFFFFFFFF,
}

/// NTFS Attribute
#[derive(Debug, Clone)]
pub struct MftAttribute {
    pub attr_type: AttributeType,
    pub length: u32,
    pub non_resident: bool,
    pub name_length: u8,
    pub name_offset: u16,
    pub flags: u16,
    pub attribute_id: u16,
    pub data: AttributeData,
}

#[derive(Debug, Clone)]
pub enum AttributeData {
    Resident { data: Vec<u8> },
    NonResident {
        start_vcn: u64,
        end_vcn: u64,
        run_offset: u16,
        compression_unit_size: u16,
        allocated_size: u64,
        data_size: u64,
        initialized_size: u64,
        data_runs: Vec<DataRun>,
    },
}

/// Data Run for non-resident attributes
#[derive(Debug, Clone)]
pub struct DataRun {
    pub cluster_offset: i64,
    pub cluster_count: u64,
}

/// File Name Attribute
#[derive(Debug, Clone)]
pub struct FileNameAttr {
    pub parent_directory: u64,
    pub creation_time: u64,
    pub modification_time: u64,
    pub mft_modification_time: u64,
    pub access_time: u64,
    pub allocated_size: u64,
    pub data_size: u64,
    pub flags: u32,
    pub reparse_value: u32,
    pub name_length: u8,
    pub namespace: u8,
    pub name: String,
}

/// Standard Information Attribute
#[derive(Debug, Clone)]
pub struct StandardInfo {
    pub creation_time: u64,
    pub modification_time: u64,
    pub mft_modification_time: u64,
    pub access_time: u64,
    pub flags: u32,
    pub max_versions: u32,
    pub version_number: u32,
    pub class_id: u32,
    pub owner_id: u32,
    pub security_id: u32,
    pub quota_charged: u64,
    pub usn: u64,
}

/// NTFS Filesystem Implementation
pub struct NtfsFilesystem {
    block_manager: BlockManager,
    boot_sector: Option<NtfsBootSector>,
    mft_cache: HashMap<u64, MftEntry>,
    cluster_size: u64,
    mft_record_size: u64,
}

impl NtfsFilesystem {
    pub fn new(block_manager: BlockManager) -> Self {
        Self {
            block_manager,
            boot_sector: None,
            mft_cache: HashMap::new(),
            cluster_size: 0,
            mft_record_size: 0,
        }
    }

    /// Parse NTFS boot sector from raw data
    pub async fn parse_boot_sector(&mut self) -> Result<(), CrossCryptError> {
        let block = self.block_manager.read_block(0).await?;
        
        // Check NTFS signature
        if &block[3..7] != b"NTFS" {
            return Err(CrossCryptError::Crypto(
                "Not an NTFS filesystem".to_string()
            ));
        }

        let boot = NtfsBootSector {
            bytes_per_sector: u16::from_le_bytes([block[11], block[12]]),
            sectors_per_cluster: block[13],
            total_sectors: u64::from_le_bytes([
                block[40], block[41], block[42], block[43],
                block[44], block[45], block[46], block[47],
            ]),
            mft_cluster: u64::from_le_bytes([
                block[48], block[49], block[50], block[51],
                block[52], block[53], block[54], block[55],
            ]),
            mft_mirror_cluster: u64::from_le_bytes([
                block[56], block[57], block[58], block[59],
                block[60], block[61], block[62], block[63],
            ]),
            clusters_per_mft_record: block[64] as i8,
            clusters_per_index_record: block[68] as i8,
            serial_number: u64::from_le_bytes([
                block[72], block[73], block[74], block[75],
                block[76], block[77], block[78], block[79],
            ]),
        };

        self.cluster_size = boot.bytes_per_sector as u64 * boot.sectors_per_cluster as u64;
        
        // Calculate MFT record size
        self.mft_record_size = if boot.clusters_per_mft_record < 0 {
            2u64.pow((-boot.clusters_per_mft_record) as u32)
        } else {
            boot.clusters_per_mft_record as u64 * self.cluster_size
        };

        debug!(
            "NTFS Boot Sector: bytes_per_sector={}, sectors_per_cluster={}, cluster_size={}, mft_record_size={}",
            boot.bytes_per_sector, boot.sectors_per_cluster, self.cluster_size, self.mft_record_size
        );

        self.boot_sector = Some(boot);
        Ok(())
    }

    /// Read MFT entry
    pub async fn read_mft_entry(&mut self, record_number: u64) -> Result<MftEntry, CrossCryptError> {
        // Check cache
        if let Some(entry) = self.mft_cache.get(&record_number) {
            return Ok(entry.clone());
        }

        let boot = self.boot_sector.as_ref()
            .ok_or_else(|| CrossCryptError::Crypto("Boot sector not parsed".to_string()))?;

        // Calculate MFT entry location
        let mft_byte_offset = boot.mft_cluster * self.cluster_size + record_number * self.mft_record_size;
        let block_number = mft_byte_offset / self.block_manager.sector_size as u64;
        let block_offset = (mft_byte_offset % self.block_manager.sector_size as u64) as usize;

        // Read blocks containing MFT entry
        let blocks_needed = (self.mft_record_size as usize + block_offset + self.block_manager.sector_size - 1) 
            / self.block_manager.sector_size;
        let data = self.block_manager.read_blocks(block_number, blocks_needed).await?;

        // Parse MFT entry
        let entry = self.parse_mft_entry(&data[block_offset..block_offset + self.mft_record_size as usize])?;
        
        self.mft_cache.insert(record_number, entry.clone());
        Ok(entry)
    }

    fn parse_mft_entry(&self, data: &[u8]) -> Result<MftEntry, CrossCryptError> {
        let signature = [data[0], data[1], data[2], data[3]];
        
        if &signature != b"FILE" {
            return Err(CrossCryptError::Crypto(
                format!("Invalid MFT entry signature: {:?}", signature)
            ));
        }

        let flags = u16::from_le_bytes([data[22], data[23]]);
        let used_size = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        let allocated_size = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);
        let base_record = u64::from_le_bytes([
            data[32], data[33], data[34], data[35],
            data[36], data[37], data[38], data[39],
        ]);
        let next_attribute_id = u16::from_le_bytes([data[40], data[41]]);

        // Parse attributes
        let mut attributes = Vec::new();
        let mut offset = u16::from_le_bytes([data[20], data[21]]) as usize;

        loop {
            if offset + 4 > data.len() {
                break;
            }

            let attr_type = u32::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            ]);

            if attr_type == 0xFFFFFFFF {
                break;
            }

            if attr_type == 0 {
                break;
            }

            let attr = self.parse_attribute(&data[offset..])?;
            offset += attr.length as usize;
            attributes.push(attr);
        }

        Ok(MftEntry {
            signature,
            flags,
            used_size,
            allocated_size,
            base_record,
            next_attribute_id,
            attributes,
        })
    }

    fn parse_attribute(&self, data: &[u8]) -> Result<MftAttribute, CrossCryptError> {
        let attr_type = match u32::from_le_bytes([data[0], data[1], data[2], data[3]]) {
            0x10 => AttributeType::StandardInformation,
            0x20 => AttributeType::AttributeList,
            0x30 => AttributeType::FileName,
            0x40 => AttributeType::ObjectId,
            0x50 => AttributeType::SecurityDescriptor,
            0x60 => AttributeType::VolumeName,
            0x70 => AttributeType::VolumeInformation,
            0x80 => AttributeType::Data,
            0x90 => AttributeType::IndexRoot,
            0xA0 => AttributeType::IndexAllocation,
            0xB0 => AttributeType::Bitmap,
            0xC0 => AttributeType::ReparsePoint,
            0xD0 => AttributeType::EaInformation,
            0xE0 => AttributeType::Ea,
            0x100 => AttributeType::LoggedUtilityStream,
            0xFFFFFFFF => AttributeType::End,
            _ => return Err(CrossCryptError::Crypto("Unknown attribute type".to_string())),
        };

        let length = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let non_resident = data[8] != 0;
        let name_length = data[9];
        let name_offset = u16::from_le_bytes([data[10], data[11]]);
        let flags = u16::from_le_bytes([data[12], data[13]]);
        let attribute_id = u16::from_le_bytes([data[14], data[15]]);

        let data = if non_resident {
            self.parse_non_resident_data(&data[16..length as usize])?
        } else {
            let value_length = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
            let value_offset = u16::from_le_bytes([data[20], data[21]]) as usize;
            AttributeData::Resident {
                data: data[value_offset..value_offset + value_length as usize].to_vec(),
            }
        };

        Ok(MftAttribute {
            attr_type,
            length,
            non_resident,
            name_length,
            name_offset,
            flags,
            attribute_id,
            data,
        })
    }

    fn parse_non_resident_data(&self, data: &[u8]) -> Result<AttributeData, CrossCryptError> {
        let start_vcn = u64::from_le_bytes([
            data[0], data[1], data[2], data[3],
            data[4], data[5], data[6], data[7],
        ]);
        let end_vcn = u64::from_le_bytes([
            data[8], data[9], data[10], data[11],
            data[12], data[13], data[14], data[15],
        ]);
        let run_offset = u16::from_le_bytes([data[16], data[17]]);
        let compression_unit_size = u16::from_le_bytes([data[18], data[19]]);
        let allocated_size = u64::from_le_bytes([
            data[24], data[25], data[26], data[27],
            data[28], data[29], data[30], data[31],
        ]);
        let data_size = u64::from_le_bytes([
            data[32], data[33], data[34], data[35],
            data[36], data[37], data[38], data[39],
        ]);
        let initialized_size = u64::from_le_bytes([
            data[40], data[41], data[42], data[43],
            data[44], data[45], data[46], data[47],
        ]);

        // Parse data runs
        let mut data_runs = Vec::new();
        let mut offset = run_offset as usize;
        let mut current_cluster = 0i64;

        loop {
            if offset >= data.len() || data[offset] == 0 {
                break;
            }

            let header = data[offset];
            let offset_size = (header & 0x0F) as usize;
            let length_size = ((header >> 4) & 0x0F) as usize;
            offset += 1;

            let cluster_count = self.parse_variable_int(&data[offset..offset + length_size])?;
            offset += length_size;

            let cluster_offset = self.parse_signed_variable_int(&data[offset..offset + offset_size])?;
            offset += offset_size;

            current_cluster += cluster_offset;

            data_runs.push(DataRun {
                cluster_offset: current_cluster,
                cluster_count,
            });
        }

        Ok(AttributeData::NonResident {
            start_vcn,
            end_vcn,
            run_offset,
            compression_unit_size,
            allocated_size,
            data_size,
            initialized_size,
            data_runs,
        })
    }

    fn parse_variable_int(&self, data: &[u8]) -> Result<u64, CrossCryptError> {
        let mut result = 0u64;
        for (i, &byte) in data.iter().enumerate() {
            result |= (byte as u64) << (i * 8);
        }
        Ok(result)
    }

    fn parse_signed_variable_int(&self, data: &[u8]) -> Result<i64, CrossCryptError> {
        let mut result = 0i64;
        for (i, &byte) in data.iter().enumerate() {
            result |= (byte as i64) << (i * 8);
        }
        
        // Sign extend if highest bit is set
        if !data.is_empty() && (data[data.len() - 1] & 0x80) != 0 {
            for i in data.len()..8 {
                result |= 0xFFi64 << (i * 8);
            }
        }
        
        Ok(result)
    }

    /// Parse file name attribute
    pub fn parse_file_name(data: &[u8]) -> Result<FileNameAttr, CrossCryptError> {
        let parent_directory = u64::from_le_bytes([
            data[0], data[1], data[2], data[3],
            data[4], data[5], data[6], data[7],
        ]);
        let creation_time = u64::from_le_bytes([
            data[8], data[9], data[10], data[11],
            data[12], data[13], data[14], data[15],
        ]);
        let modification_time = u64::from_le_bytes([
            data[16], data[17], data[18], data[19],
            data[20], data[21], data[22], data[23],
        ]);
        let mft_modification_time = u64::from_le_bytes([
            data[24], data[25], data[26], data[27],
            data[28], data[29], data[30], data[31],
        ]);
        let access_time = u64::from_le_bytes([
            data[32], data[33], data[34], data[35],
            data[36], data[37], data[38], data[39],
        ]);
        let allocated_size = u64::from_le_bytes([
            data[40], data[41], data[42], data[43],
            data[44], data[45], data[46], data[47],
        ]);
        let data_size = u64::from_le_bytes([
            data[48], data[49], data[50], data[51],
            data[52], data[53], data[54], data[55],
        ]);
        let flags = u32::from_le_bytes([data[56], data[57], data[58], data[59]]);
        let reparse_value = u32::from_le_bytes([data[60], data[61], data[62], data[63]]);
        let name_length = data[64];
        let namespace = data[65];

        // Parse UTF-16LE name
        let name_data = &data[66..66 + name_length as usize * 2];
        let name = String::from_utf16_lossy(
            &name_data.chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect::<Vec<_>>()
        );

        Ok(FileNameAttr {
            parent_directory,
            creation_time,
            modification_time,
            mft_modification_time,
            access_time,
            allocated_size,
            data_size,
            flags,
            reparse_value,
            name_length,
            namespace,
            name,
        })
    }

    /// Convert NTFS time to SystemTime
    pub fn ntfs_time_to_systemtime(ntfs_time: u64) -> SystemTime {
        // NTFS time is 100-nanosecond intervals since January 1, 1601
        // Unix epoch is January 1, 1970
        // Difference is 11644473600 seconds
        let unix_time = (ntfs_time / 10_000_000) as i64 - 11644473600i64;
        
        if unix_time < 0 {
            SystemTime::UNIX_EPOCH
        } else {
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(unix_time as u64)
        }
    }
}

#[async_trait::async_trait]
impl EncryptFileSystem for NtfsFilesystem {
    async fn init(&self) -> Result<(), CrossCryptError> {
        debug!("Initializing NTFS filesystem");
        Ok(())
    }

    async fn destroy(&self) {
        debug!("Destroying NTFS filesystem");
    }

    async fn lookup(&self, parent: u64, name: &std::ffi::OsStr) -> Result<FileAttr, CrossCryptError> {
        todo!("Implement NTFS lookup")
    }

    async fn getattr(&self, ino: u64) -> Result<FileAttr, CrossCryptError> {
        todo!("Implement NTFS getattr")
    }

    async fn setattr(&self, _ino: u64, _attr: FileAttr) -> Result<FileAttr, CrossCryptError> {
        Err(CrossCryptError::Crypto("Read-only filesystem".to_string()))
    }

    async fn readlink(&self, _ino: u64) -> Result<String, CrossCryptError> {
        Err(CrossCryptError::Crypto("Not a symlink".to_string()))
    }

    async fn mknod(&self, _parent: u64, _name: &std::ffi::OsStr, _mode: u32, _rdev: u32) -> Result<FileAttr, CrossCryptError> {
        Err(CrossCryptError::Crypto("Read-only filesystem".to_string()))
    }

    async fn mkdir(&self, _parent: u64, _name: &std::ffi::OsStr, _mode: u32) -> Result<FileAttr, CrossCryptError> {
        Err(CrossCryptError::Crypto("Read-only filesystem".to_string()))
    }

    async fn unlink(&self, _parent: u64, _name: &std::ffi::OsStr) -> Result<(), CrossCryptError> {
        Err(CrossCryptError::Crypto("Read-only filesystem".to_string()))
    }

    async fn rmdir(&self, _parent: u64, _name: &std::ffi::OsStr) -> Result<(), CrossCryptError> {
        Err(CrossCryptError::Crypto("Read-only filesystem".to_string()))
    }

    async fn symlink(&self, _parent: u64, _name: &std::ffi::OsStr, _link: &std::ffi::OsStr) -> Result<FileAttr, CrossCryptError> {
        Err(CrossCryptError::Crypto("Read-only filesystem".to_string()))
    }

    async fn rename(&self, _parent: u64, _name: &std::ffi::OsStr, _newparent: u64, _newname: &std::ffi::OsStr) -> Result<(), CrossCryptError> {
        Err(CrossCryptError::Crypto("Read-only filesystem".to_string()))
    }

    async fn link(&self, _ino: u64, _newparent: u64, _newname: &std::ffi::OsStr) -> Result<FileAttr, CrossCryptError> {
        Err(CrossCryptError::Crypto("Read-only filesystem".to_string()))
    }

    async fn open(&self, _ino: u64, _flags: u32) -> Result<u64, CrossCryptError> {
        Ok(0) // Simple implementation
    }

    async fn read(&self, _ino: u64, _fh: u64, _offset: u64, _size: u32) -> Result<Vec<u8>, CrossCryptError> {
        todo!("Implement NTFS read")
    }

    async fn write(&self, _ino: u64, _fh: u64, _offset: u64, _data: &[u8]) -> Result<u32, CrossCryptError> {
        Err(CrossCryptError::Crypto("Read-only filesystem".to_string()))
    }

    async fn flush(&self, _ino: u64, _fh: u64) -> Result<(), CrossCryptError> {
        Ok(())
    }

    async fn release(&self, _ino: u64, _fh: u64) -> Result<(), CrossCryptError> {
        Ok(())
    }

    async fn fsync(&self, _ino: u64, _datasync: bool) -> Result<(), CrossCryptError> {
        Ok(())
    }

    async fn opendir(&self, _ino: u64) -> Result<u64, CrossCryptError> {
        Ok(0)
    }

    async fn readdir(&self, _ino: u64, _fh: u64, _offset: u64) -> Result<Vec<(u64, FileType, String)>, CrossCryptError> {
        todo!("Implement NTFS readdir")
    }

    async fn releasedir(&self, _ino: u64, _fh: u64) -> Result<(), CrossCryptError> {
        Ok(())
    }

    async fn statfs(&self) -> Result<Statfs, CrossCryptError> {
        let boot = self.boot_sector.as_ref()
            .ok_or_else(|| CrossCryptError::Crypto("Boot sector not parsed".to_string()))?;

        let total_clusters = boot.total_sectors / boot.sectors_per_cluster as u64;
        
        Ok(Statfs {
            blocks: total_clusters,
            bfree: 0, // Would need to parse bitmap
            bavail: 0,
            files: 0, // Would need to count MFT entries
            ffree: 0,
            bsize: self.cluster_size as u32,
            namelen: 255,
            frsize: self.cluster_size as u32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ntfs_time_conversion() {
        let ntfs_time = 132000000000000000u64; // Some NTFS time
        let system_time = NtfsFilesystem::ntfs_time_to_systemtime(ntfs_time);
        
        // Just verify it doesn't panic
        assert!(system_time >= SystemTime::UNIX_EPOCH);
    }
}
