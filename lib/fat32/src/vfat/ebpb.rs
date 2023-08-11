use core::fmt;
use shim::const_assert_size;
use shim::io;

use crate::traits::BlockDevice;
use crate::vfat::Error;
use crate::vfat::Cluster;

// Signature that needs to be found at the end of an EBPB.
const VALID_BOOTABLE_SIGNATURE: u16 = 0xAA55;

/// Represents Extended Bios Parameter Block found on a FAT32
/// filesystem.
#[repr(C, packed)]
pub struct BiosParameterBlock {
    _machine_code: [u8; 3],
    oem_id: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    num_reserved_sectors: u16,
    num_fats: u8,
    max_dir_entries: u16,
    total_logical_sectors: u16,
    fat_id: u8,
    _deprecated: u16,
    sectors_per_track: u16,
    num_heads: u16,
    num_hidden_sectors: u32,
    total_logical_sector_overflow: u32,
    sectors_per_fat: u32,
    flags: u16,
    fat_version: [u8; 2],
    root_cluster: Cluster,
    fs_info_sector: u16,
    backup_boot_sector: u16,
    _reserved: [u8; 12],
    drive_num: u8,
    _windows_nt_flag: u8,
    signature: u8,
    serial_num: u32,
    volume_label: [u8; 11],
    system_id: [u8; 8],
    boot_code: [u8; 420],
    bootable_signature: u16,
}

const_assert_size!(BiosParameterBlock, 512);

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(mut device: T, sector: u64) -> Result<BiosParameterBlock, Error> {
        let mut ebpb_buf: [u8; 512] = [0; 512];
        let bytes_read = device.read_sector(sector, &mut ebpb_buf)?;

        if bytes_read < 512 {
            return Err(Error::from(io::Error::from(io::ErrorKind::UnexpectedEof)));
        }

        let ebpb: BiosParameterBlock = unsafe { core::mem::transmute(ebpb_buf) };

        if ebpb.bootable_signature != VALID_BOOTABLE_SIGNATURE {
            return Err(Error::BadSignature);
        }

        Ok(ebpb)
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BiosParameterBlock")
            .field("bytes_per_sector", &{ self.bytes_per_sector} )
            .field("sectors_per_cluster", &self.sectors_per_cluster)
            .field("num_reserved_sectors", &{ self.num_reserved_sectors })
            .field("num_fats", &self.num_fats)
            .field("max_dir_entries", &{ self.max_dir_entries })
            .field("total_logical_sectors", &{ self.total_logical_sectors })
            .field("fat_id", &self.fat_id)
            .field("sectors_per_track", &{ self.sectors_per_track })
            .field("num_heads", &{ self.num_heads })
            .field("num_hidden_sectors", &{ self.num_hidden_sectors })
            .field("total_logical_sector_overflow", &{ self.total_logical_sector_overflow })
            .field("sectors_per_fat", &{ self.sectors_per_fat })
            .field("flags", &{ self.flags })
            .field("fat_version", &{ self.fat_version })
            .field("root_cluster", &{ self.root_cluster })
            .field("fs_info_sector", &{ self.fs_info_sector })
            .field("backup_boot_sector", &{ self.backup_boot_sector })
            .field("drive_num", &self.drive_num)
            .field("signature", &self.signature)
            .field("serial_num", &{ self.serial_num })
            .field("volume_label", &self.volume_label)
            .field("system_id", &self.system_id)
            .field("bootable_signature", &{ self.bootable_signature })
            .finish()
    }
}
