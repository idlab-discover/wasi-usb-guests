fn main() -> Result<(), rusb::Error> {
    let devices = rusb::devices()?;

    for device in devices.iter() {
        let descriptor = device.device_descriptor()?;

        println!(
            "Bus {:03} Device {:03}: ID {:04x}:{:04x}",
            device.bus_number(),
            device.address(),
            descriptor.vendor_id(),
            descriptor.product_id(),
        );
    }

    Ok(())
}
