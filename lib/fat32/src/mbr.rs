use core::fmt;
use shim::const_assert_size;
use shim::io;

use crate::traits::BlockDevice;

/// Represents cylinder/head/sector offset data for an MBR partition entry.
/// This data is not currently used in our implementation.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct CHS {
    // Essentially a placeholder. We're not using these for anything,
    // but its important that we pad this size.
    chs_bytes: [u8; 3],
}

impl fmt::Debug for CHS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CHS")
            .field("byte 0", &self.chs_bytes[0])
            .field("byte 1", &self.chs_bytes[1])
            .field("byte 2", &self.chs_bytes[2])
            .finish()
    }
}

const_assert_size!(CHS, 3);

// Flags indicating the active status of a partition entry
const ACTIVE_PART_FLAG: u8 = 0x80;
const INACTIVE_PARTFLAG: u8 = 0x00;

/// Metadata about an entry in the MBR partition table
#[repr(C, packed)]
pub struct PartitionEntry {
    boot_indicator: u8,
    starting_chs: CHS,
    partition_type: u8,
    ending_chs: CHS,
    sector_offset: u32,
    total_sectors: u32,
}

impl fmt::Debug for PartitionEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PartitionEntry")
            .field("boot_indicator", &self.boot_indicator)
            .field("starting_chs", &self.starting_chs)
            .field("partition_type", &self.partition_type)
            .field("ending_chs", &self.ending_chs)
            .field("sector_offset", &{ self.sector_offset })
            .field("total_sectors", &{ self.total_sectors })
            .finish()
    }
}

const_assert_size!(PartitionEntry, 16);

// The "magic" two byte signature that indicates a valid MBR bootsector
const MBR_SIGNATURE: [u8; 2] = [0x55, 0xAA];

/// The master boot record (MBR).
#[repr(C, packed)]
pub struct MasterBootRecord {
    bootstrap: [u8; 436],
    disk_id: [u8; 10],
    partition_table: [PartitionEntry; 4],
    magic: [u8; 2],
}

impl fmt::Debug for MasterBootRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MasterBootRecord")
            .field("disk_id", &self.disk_id)
            .field("partition_table", &self.partition_table)
            .field("magic", &self.magic)
            .finish()
    }
}

const_assert_size!(MasterBootRecord, 512);

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partition `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        let mut mbr_buf: [u8; 512] = [0; 512];
        let bytes_read = device.read_sector(0, &mut mbr_buf)?;

        if bytes_read < 512 {
            return Err(Error::from(io::Error::from(io::ErrorKind::UnexpectedEof)));
        }

        // We've taken care of alignmenment by packing our structs and guaranteed that
        // 512 bytes have been read. This is safe to transmute.
        let mbr: MasterBootRecord = unsafe { core::mem::transmute(mbr_buf) };

        if mbr.magic != MBR_SIGNATURE {
            return Err(Error::BadSignature);
        }

        for (i, partition) in mbr.partition_table.iter().enumerate() {
            if partition.boot_indicator != ACTIVE_PART_FLAG && partition.boot_indicator != INACTIVE_PARTFLAG {
                return Err(Error::UnknownBootIndicator(i as u8));
            }
        }

        Ok(mbr)
    }
}
