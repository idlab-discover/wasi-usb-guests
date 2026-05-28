use anyhow::{Context, anyhow, bail};
use log::{debug, info};
use rusb::UsbContext;

use crate::scsi::ScsiTransport;

const MBR_SIGNATURE: u16 = 0xAA55;

fn is_fat_partition_type(ptype: u8) -> bool {
    matches!(
        ptype,
        0x01   // FAT12
        | 0x04 // FAT16 < 32 MiB
        | 0x06 // FAT16 >= 32 MiB
        | 0x0B // FAT32 (CHS)
        | 0x0C // FAT32 (LBA)
        | 0x0E // FAT16 (LBA)
    )
}

struct MbrPartitionEntry {
    partition_type: u8,
    start_lba: u32,
    size_sectors: u32,
}

fn parse_mbr_partitions(sector: &[u8]) -> Option<Vec<MbrPartitionEntry>> {
    if sector.len() < 512 {
        return None;
    }
    let sig = u16::from_le_bytes([sector[510], sector[511]]);
    if sig != MBR_SIGNATURE {
        return None;
    }
    let mut entries = Vec::new();
    for i in 0..4 {
        let base = 446 + i * 16;
        let ptype = sector[base + 4];
        if ptype == 0 {
            continue;
        }
        let start_lba = u32::from_le_bytes([
            sector[base + 8],
            sector[base + 9],
            sector[base + 10],
            sector[base + 11],
        ]);
        let size_sectors = u32::from_le_bytes([
            sector[base + 12],
            sector[base + 13],
            sector[base + 14],
            sector[base + 15],
        ]);
        entries.push(MbrPartitionEntry {
            partition_type: ptype,
            start_lba,
            size_sectors,
        });
    }
    Some(entries)
}

pub(crate) fn find_partition_start<T: UsbContext>(
    transport: &mut ScsiTransport<T>,
    block_size: u32,
) -> anyhow::Result<u32> {
    let mut sector0 = vec![0u8; block_size as usize];
    transport
        .read_sectors(0, 1, block_size, &mut sector0)
        .context("Failed to read sector 0")?;

    let entries =
        parse_mbr_partitions(&sector0).ok_or_else(|| anyhow!("Sector 0 is not a valid MBR"))?;

    info!("MBR partition table found ({} entries)", entries.len());
    for (i, entry) in entries.iter().enumerate() {
        debug!(
            "  Partition {}: type=0x{:02X}, start_lba={}, size={} sectors",
            i + 1,
            entry.partition_type,
            entry.start_lba,
            entry.size_sectors,
        );
    }

    for entry in &entries {
        if is_fat_partition_type(entry.partition_type) {
            info!(
                "Using partition at LBA {} (type 0x{:02X})",
                entry.start_lba, entry.partition_type
            );
            return Ok(entry.start_lba);
        }
    }

    bail!(
        "No FAT partition found in MBR (partition types: {:?})",
        entries
            .iter()
            .map(|e| format!("0x{:02X}", e.partition_type))
            .collect::<Vec<_>>()
    );
}
