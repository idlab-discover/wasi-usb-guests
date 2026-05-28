//! SCSI command transport over USB Mass Storage Bulk-Only Transport.
//!
//! https://www.seagate.com/files/staticfiles/support/docs/manual/Interface%20manuals/100293068j.pdf

use anyhow::{Context, bail};
use log::{debug, trace};
use rusb::UsbContext;
use std::time::Duration;

use crate::bot::{self, CSW_SIZE, Csw};

const SCSI_TEST_UNIT_READY: u8 = 0x00;
const SCSI_INQUIRY: u8 = 0x12;
const SCSI_READ_CAPACITY_10: u8 = 0x25;
const SCSI_READ_10: u8 = 0x28;

const USB_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) struct ScsiTransport<T: UsbContext> {
    handle: rusb::DeviceHandle<T>,
    endpoint_in: u8,
    endpoint_out: u8,
    tag: u32,
}

impl<T: UsbContext> ScsiTransport<T> {
    pub fn new(handle: rusb::DeviceHandle<T>, endpoint_in: u8, endpoint_out: u8) -> Self {
        Self {
            handle,
            endpoint_in,
            endpoint_out,
            tag: 1,
        }
    }

    /// Execute a SCSI command using BOT (CBW -> optional data phase -> CSW)
    fn execute(&mut self, cb: &[u8], data_in_len: u32) -> anyhow::Result<Vec<u8>> {
        let tag = self.tag;
        self.tag = self.tag.wrapping_add(1);

        let flags = if data_in_len > 0 {
            bot::FLAG_DATA_IN
        } else {
            bot::FLAG_DATA_OUT
        };

        // Send CBW
        let cbw = bot::build_cbw(tag, data_in_len, flags, 0, cb);
        trace!(
            "Sending CBW: tag={}, data_len={}, opcode=0x{:02X}",
            tag, data_in_len, cb[0]
        );
        self.handle
            .write_bulk(self.endpoint_out, &cbw, USB_TIMEOUT)
            .context("Failed to send CBW")?;

        // Data phase (IN only)
        let mut data = Vec::new();
        if data_in_len > 0 {
            data.resize(data_in_len as usize, 0u8);
            let n = self
                .handle
                .read_bulk(self.endpoint_in, &mut data, USB_TIMEOUT)
                .context("Failed to read data phase")?;
            data.truncate(n);
            trace!("Data phase: received {} bytes", n);
        }

        // Receive CSW
        let mut csw_buf = [0u8; CSW_SIZE];
        self.handle
            .read_bulk(self.endpoint_in, &mut csw_buf, USB_TIMEOUT)
            .context("Failed to read CSW")?;
        let csw = Csw::parse(&csw_buf)?;

        if csw.tag != tag {
            bail!("CSW tag mismatch: expected {}, got {}", tag, csw.tag);
        }
        if csw.status != 0 {
            bail!(
                "SCSI command failed: CSW status={}, residue={}",
                csw.status,
                csw.data_residue
            );
        }
        trace!("CSW: status=0, residue={}", csw.data_residue);

        Ok(data)
    }

    /// SCSI INQUIRY
    pub fn inquiry(&mut self) -> anyhow::Result<(String, String)> {
        debug!("Sending SCSI INQUIRY");
        let alloc_len: u8 = 36;
        let cdb = [SCSI_INQUIRY, 0, 0, 0, alloc_len, 0];
        let data = self.execute(&cdb, alloc_len as u32)?;

        if data.len() < 36 {
            bail!("INQUIRY response too short ({} bytes)", data.len());
        }
        let vendor = String::from_utf8_lossy(&data[8..16]).trim().to_string();
        let product = String::from_utf8_lossy(&data[16..32]).trim().to_string();
        debug!("INQUIRY result: vendor={:?}, product={:?}", vendor, product);
        Ok((vendor, product))
    }

    /// SCSI TEST UNIT READY
    pub fn test_unit_ready(&mut self, retries: u32) -> anyhow::Result<()> {
        debug!("Sending SCSI TEST UNIT READY (max {} attempts)", retries);
        let cdb = [SCSI_TEST_UNIT_READY, 0, 0, 0, 0, 0];
        for attempt in 0..retries {
            match self.execute(&cdb, 0) {
                Ok(_) => {
                    debug!("Device ready on attempt {}", attempt + 1);
                    return Ok(());
                }
                Err(e) => {
                    if attempt + 1 == retries {
                        return Err(e).context("Device not ready after retries");
                    }
                    debug!("Attempt {} failed, retrying...", attempt + 1);
                    std::thread::sleep(Duration::from_millis(500));
                }
            }
        }
        unreachable!()
    }

    /// SCSI READ CAPACITY (10)
    pub fn read_capacity(&mut self) -> anyhow::Result<(u32, u32)> {
        debug!("Sending SCSI READ CAPACITY (10)");
        let cdb = [SCSI_READ_CAPACITY_10, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let data = self.execute(&cdb, 8)?;

        if data.len() < 8 {
            bail!("READ CAPACITY response too short ({} bytes)", data.len());
        }
        let last_lba = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let block_size = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        debug!(
            "READ CAPACITY: last_lba={}, block_size={}",
            last_lba, block_size
        );
        Ok((last_lba, block_size))
    }

    /// SCSI READ (10)
    /// reads `count` sectors starting at `lba` into `buf`
    pub fn read_sectors(
        &mut self,
        lba: u32,
        count: u16,
        block_size: u32,
        buf: &mut [u8],
    ) -> anyhow::Result<usize> {
        let transfer_len = count as u32 * block_size;
        assert!(buf.len() >= transfer_len as usize);

        let lba_bytes = lba.to_be_bytes();
        let count_bytes = count.to_be_bytes();
        let cdb = [
            SCSI_READ_10,
            0,
            lba_bytes[0],
            lba_bytes[1],
            lba_bytes[2],
            lba_bytes[3],
            0,
            count_bytes[0],
            count_bytes[1],
            0,
        ];

        trace!("READ(10): lba={}, count={}", lba, count);
        let data = self.execute(&cdb, transfer_len)?;
        let n = data.len().min(buf.len());
        buf[..n].copy_from_slice(&data[..n]);
        Ok(n)
    }
}
