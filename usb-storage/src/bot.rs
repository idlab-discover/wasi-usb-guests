//! USB Mass Storage Bulk-Only Transport (BOT) protocol.
//!
//! Command Block Wrapper (CBW) serialization and Command Status Wrapper (CSW) parsing per the USB Mass Storage Class Bulk-Only Transport specification.
//! https://wiki.osdev.org/USB_Mass_Storage_Class_Devices

use anyhow::bail;

const CBW_SIGNATURE: u32 = 0x43425355;
const CSW_SIGNATURE: u32 = 0x53425355;

pub(crate) const CBW_SIZE: usize = 31;
pub(crate) const CSW_SIZE: usize = 13;

pub(crate) const FLAG_DATA_IN: u8 = 0x80;
pub(crate) const FLAG_DATA_OUT: u8 = 0x00;

pub(crate) fn build_cbw(tag: u32, data_len: u32, flags: u8, lun: u8, cb: &[u8]) -> [u8; CBW_SIZE] {
    let mut cbw = [0u8; CBW_SIZE];
    cbw[0..4].copy_from_slice(&CBW_SIGNATURE.to_le_bytes());
    cbw[4..8].copy_from_slice(&tag.to_le_bytes());
    cbw[8..12].copy_from_slice(&data_len.to_le_bytes());
    cbw[12] = flags;
    cbw[13] = lun & 0x0F;
    cbw[14] = cb.len() as u8;
    let copy_len = cb.len().min(16);
    cbw[15..15 + copy_len].copy_from_slice(&cb[..copy_len]);
    cbw
}

pub(crate) struct Csw {
    pub tag: u32,
    pub data_residue: u32,
    pub status: u8,
}

impl Csw {
    pub fn parse(buf: &[u8; CSW_SIZE]) -> anyhow::Result<Self> {
        let sig = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if sig != CSW_SIGNATURE {
            bail!("Invalid CSW signature: 0x{:08X}", sig);
        }
        Ok(Csw {
            tag: u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
            data_residue: u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]),
            status: buf[12],
        })
    }
}
