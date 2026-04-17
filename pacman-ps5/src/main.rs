use anyhow::anyhow;
use pacman::run_game_loop;
use pacman::types::GamepadState;
use rusb::{Context, Direction, UsbContext};
use std::time::Duration;

pub fn parse_ps5_controller_data(data: &[u8]) -> GamepadState {
    if data.len() < 10 {
        return GamepadState::default();
    }

    // PS5 DualSense typically uses Report ID 1 over USB
    let offset = if data[0] == 0x01 {
        0
    } else {
        return GamepadState::default();
    };

    // Joystick axes (Bytes 1-4)
    let ls_x = (data[offset + 1] as f32 - 128.0) / 128.0;
    let ls_y = -((data[offset + 2] as f32 - 128.0) / 128.0);
    let rs_x = (data[offset + 3] as f32 - 128.0) / 128.0;
    let rs_y = -((data[offset + 4] as f32 - 128.0) / 128.0);

    // Triggers (Bytes 5-6)
    let lt = data[offset + 5] as f32 / 255.0;
    let rt = data[offset + 6] as f32 / 255.0;

    // D-Pad is typically a 4-bit hat switch in byte 8 (values 0-7, 8 is neutral)
    let dpad = data[offset + 8] & 0x0F;
    let up = dpad == 0 || dpad == 1 || dpad == 7;
    let right = dpad == 1 || dpad == 2 || dpad == 3;
    let down = dpad == 3 || dpad == 4 || dpad == 5;
    let left = dpad == 5 || dpad == 6 || dpad == 7;

    let square = (data[offset + 8] & 0x10) != 0;
    let cross = (data[offset + 8] & 0x20) != 0;
    let circle = (data[offset + 8] & 0x40) != 0;
    let triangle = (data[offset + 8] & 0x80) != 0;

    GamepadState {
        a: cross,    // Map cross to A
        b: circle,   // Map circle to B
        x: square,   // Map square to X
        y: triangle, // Map triangle to Y
        up,
        down,
        left,
        right,
        lt,
        rt,
        lstick_x: ls_x,
        lstick_y: ls_y,
        rstick_x: rs_x,
        rstick_y: rs_y,
        ..Default::default()
    }
}

fn main() -> anyhow::Result<()> {
    let context = Context::new()?;
    let mut handle = None;

    let ps5_ids = (0x054c, 0x0ce6);

    for device in context.devices()?.iter() {
        let desc = device.device_descriptor()?;
        if desc.vendor_id() == ps5_ids.0 && desc.product_id() == ps5_ids.1 {
            handle = Some(device.open()?);
            break;
        }
    }

    let handle = handle.ok_or_else(|| anyhow!("No supported controller PS5 found!"))?;
    let device = handle.device();
    let config_desc = device.config_descriptor(0)?;

    let mut endpoint_in = None;
    let mut _endpoint_out = None;
    let mut target_interface = None;

    // Find the interface with the interrupt IN endpoint
    for interface in config_desc.interfaces() {
        for alt_setting in interface.descriptors() {
            for ep_desc in alt_setting.endpoint_descriptors() {
                if ep_desc.transfer_type() == rusb::TransferType::Interrupt {
                    if ep_desc.direction() == Direction::In {
                        endpoint_in = Some(ep_desc.address());
                        target_interface = Some(interface.number());
                    } else if ep_desc.direction() == Direction::Out {
                        _endpoint_out = Some(ep_desc.address());
                    }
                }
            }
        }
        if target_interface.is_some() {
            break;
        }
    }

    let interface_number = target_interface
        .ok_or_else(|| anyhow!("Could not find an interface with an Interrupt IN endpoint"))?;
    let endpoint_in = endpoint_in.unwrap();

    // Detach kernel driver if needed
    match handle.kernel_driver_active(interface_number) {
        Ok(true) => {
            handle.detach_kernel_driver(interface_number)?;
        }
        Ok(false) => {}
        Err(e) => return Err(e.into()),
    }

    handle.set_auto_detach_kernel_driver(true)?;

    handle.claim_interface(interface_number)?;

    let mut buf = [0u8; 64];

    run_game_loop(|| {
        match handle.read_interrupt(endpoint_in, &mut buf, Duration::from_millis(50)) {
            Ok(bytes_read) => Some(parse_ps5_controller_data(&buf[0..bytes_read])),
            _ => None,
        }
    })?;

    Ok(())
}
