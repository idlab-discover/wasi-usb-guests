use bindings::device;

pub fn main() {
    if let Err(err) = run() {
        eprintln!("lsusb failed: {err:?}");
    }
}

fn run() -> Result<(), device::LibusbError> {
    device::init()?;

    let devices = device::list_devices()?;

    if devices.is_empty() {
        eprintln!("No USB devices found.");
        return Ok(());
    }

    for (_device, descriptor, location) in devices {
        println!(
            "Bus {:03} Device {:03}: ID {:04x}:{:04x}",
            location.bus_number,
            location.device_address,
            descriptor.vendor_id,
            descriptor.product_id
        );
    }

    Ok(())
}
