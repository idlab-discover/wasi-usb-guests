use log::trace;
use rusb::UsbContext;
use std::io::{self, Read, Seek, SeekFrom, Write};

use crate::scsi::ScsiTransport;

pub struct UsbBlockDevice<T: UsbContext> {
    transport: ScsiTransport<T>,
    block_size: u32,
    partition_sectors: u32,
    partition_start_lba: u32,
    position: u64,
    sector_cache: Vec<u8>,
    cached_lba: Option<u32>,
}

impl<T: UsbContext> UsbBlockDevice<T> {
    pub(crate) fn new(
        transport: ScsiTransport<T>,
        block_size: u32,
        disk_block_count: u32,
        partition_start_lba: u32,
    ) -> Self {
        let partition_sectors = disk_block_count.saturating_sub(partition_start_lba);
        Self {
            transport,
            block_size,
            partition_sectors,
            partition_start_lba,
            position: 0,
            sector_cache: vec![0u8; block_size as usize],
            cached_lba: None,
        }
    }

    fn total_size(&self) -> u64 {
        self.partition_sectors as u64 * self.block_size as u64
    }

    fn ensure_cached(&mut self, abs_lba: u32) -> io::Result<()> {
        if self.cached_lba == Some(abs_lba) {
            return Ok(());
        }
        trace!("Cache miss: reading sector {}", abs_lba);
        self.transport
            .read_sectors(abs_lba, 1, self.block_size, &mut self.sector_cache)
            .map_err(io::Error::other)?;
        self.cached_lba = Some(abs_lba);
        Ok(())
    }
}

impl<T: UsbContext> Read for UsbBlockDevice<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() || self.position >= self.total_size() {
            return Ok(0);
        }

        let bs = self.block_size as u64;
        let rel_lba = (self.position / bs) as u32;
        let offset_in_sector = (self.position % bs) as usize;
        let abs_lba = self.partition_start_lba + rel_lba;

        self.ensure_cached(abs_lba)?;

        let available = self.block_size as usize - offset_in_sector;
        let to_copy = buf.len().min(available);
        buf[..to_copy]
            .copy_from_slice(&self.sector_cache[offset_in_sector..offset_in_sector + to_copy]);
        self.position += to_copy as u64;
        Ok(to_copy)
    }
}

impl<T: UsbContext> Write for UsbBlockDevice<T> {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "read-only block device",
        ))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<T: UsbContext> Seek for UsbBlockDevice<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos: i64 = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::Current(offset) => self.position as i64 + offset,
            SeekFrom::End(offset) => self.total_size() as i64 + offset,
        };
        if new_pos < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek to negative position",
            ));
        }
        self.position = new_pos as u64;
        Ok(self.position)
    }
}
