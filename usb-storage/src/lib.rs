mod bot;
mod discovery;
mod partition;
mod scsi;

pub mod block_device;

pub use fatfs;

use anyhow::Context;
use log::info;

use block_device::UsbBlockDevice;
use scsi::ScsiTransport;

pub type FileSystem = fatfs::FileSystem<UsbBlockDevice<rusb::Context>>;

pub fn mount() -> anyhow::Result<FileSystem> {
    let context = rusb::Context::new()?;
    let msd = discovery::find_mass_storage_device(&context)?;

    let desc = msd.device.device_descriptor()?;
    info!(
        "Found mass storage device: VID={:04x} PID={:04x}, interface={}, bulk_in=0x{:02X}, bulk_out=0x{:02X}",
        desc.vendor_id(),
        desc.product_id(),
        msd.interface_number,
        msd.endpoint_in,
        msd.endpoint_out,
    );

    let handle = msd.device.open().context("Failed to open device")?;

    match handle.kernel_driver_active(msd.interface_number) {
        Ok(true) => {
            info!(
                "Detaching kernel driver from interface {}",
                msd.interface_number
            );
            handle
                .detach_kernel_driver(msd.interface_number)
                .context("Failed to detach kernel driver")?;
        }
        Ok(false) => {}
        Err(_) => {}
    }

    handle
        .claim_interface(msd.interface_number)
        .context("Failed to claim interface")?;
    info!("Interface {} claimed", msd.interface_number);

    let mut transport = ScsiTransport::new(handle, msd.endpoint_in, msd.endpoint_out);

    let (vendor, product) = transport.inquiry().context("INQUIRY failed")?;
    info!("SCSI device: vendor={:?}, product={:?}", vendor, product);

    transport
        .test_unit_ready(10)
        .context("TEST UNIT READY failed")?;
    info!("Device is ready");

    let (last_lba, block_size) = transport.read_capacity().context("READ CAPACITY failed")?;
    let block_count = last_lba + 1;
    let total_mb = (block_count as u64 * block_size as u64) / (1024 * 1024);
    info!(
        "Disk capacity: {} sectors x {} bytes = {} MiB",
        block_count, block_size, total_mb
    );

    let partition_start_lba = partition::find_partition_start(&mut transport, block_size)?;

    info!("Mounting FAT filesystem...");
    let block_device = UsbBlockDevice::new(transport, block_size, block_count, partition_start_lba);
    let fs = fatfs::FileSystem::new(block_device, fatfs::FsOptions::new())
        .context("Failed to mount FAT filesystem")?;

    info!("Filesystem type: {:?}", fs.fat_type());
    info!("Volume label: {}", fs.volume_label());

    Ok(fs)
}
