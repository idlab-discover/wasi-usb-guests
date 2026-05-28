//! USB mass storage device discovery.
//!
//! Scans the USB bus for a device exposing the Mass Storage class with SCSI transparent command set over Bulk-Only Transport (class 0x08, subclass 0x06, protocol 0x50).

use anyhow::anyhow;
use log::debug;
use rusb::{Context, Direction, TransferType, UsbContext};

const USB_CLASS_MASS_STORAGE: u8 = 0x08;
const USB_SUBCLASS_SCSI: u8 = 0x06;
const USB_PROTOCOL_BOT: u8 = 0x50;

pub(crate) struct MassStorageDevice<T: UsbContext> {
    pub device: rusb::Device<T>,
    pub interface_number: u8,
    pub endpoint_in: u8,
    pub endpoint_out: u8,
}

/// Enumerate USB devices and return the first mass storage device using BOT
pub(crate) fn find_mass_storage_device(
    context: &Context,
) -> anyhow::Result<MassStorageDevice<Context>> {
    for device in context.devices()?.iter() {
        let desc = device.device_descriptor()?;
        debug!(
            "Scanning device VID={:04x} PID={:04x}",
            desc.vendor_id(),
            desc.product_id()
        );

        for cfg_idx in 0..desc.num_configurations() {
            let config = match device.config_descriptor(cfg_idx) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for interface in config.interfaces() {
                for alt in interface.descriptors() {
                    if alt.class_code() == USB_CLASS_MASS_STORAGE
                        && alt.sub_class_code() == USB_SUBCLASS_SCSI
                        && alt.protocol_code() == USB_PROTOCOL_BOT
                    {
                        let mut ep_in = None;
                        let mut ep_out = None;
                        for ep in alt.endpoint_descriptors() {
                            if ep.transfer_type() == TransferType::Bulk {
                                match ep.direction() {
                                    Direction::In => ep_in = Some(ep.address()),
                                    Direction::Out => ep_out = Some(ep.address()),
                                }
                            }
                        }
                        if let (Some(ei), Some(eo)) = (ep_in, ep_out) {
                            debug!(
                                "Found mass storage: iface={}, ep_in=0x{:02X}, ep_out=0x{:02X}",
                                interface.number(),
                                ei,
                                eo
                            );
                            return Ok(MassStorageDevice {
                                device,
                                interface_number: interface.number(),
                                endpoint_in: ei,
                                endpoint_out: eo,
                            });
                        }
                    }
                }
            }
        }
    }

    Err(anyhow!("No USB mass storage device found"))
}
