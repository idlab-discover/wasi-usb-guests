use anyhow::anyhow;
use byteorder::ByteOrder;
use pacman::run_game_loop;
use pacman::types::GamepadState;
use std::time::Duration;

pub fn parse_xbox_controller_data(data: &[u8]) -> GamepadState {
    if data.len() < 10 {
        return GamepadState::default();
    }

    let lt = byteorder::LittleEndian::read_u16(&data[6..]) as f32 / 1023.0;
    let rt = byteorder::LittleEndian::read_u16(&data[8..]) as f32 / 1023.0;

    let lstick_x = (byteorder::LittleEndian::read_i16(&data[10..]) as f32 + 0.5) / 32767.5;
    let lstick_y = (byteorder::LittleEndian::read_i16(&data[12..]) as f32 + 0.5) / 32767.5;
    let rstick_x = (byteorder::LittleEndian::read_i16(&data[14..]) as f32 + 0.5) / 32767.5;
    let rstick_y = (byteorder::LittleEndian::read_i16(&data[16..]) as f32 + 0.5) / 32767.5;

    GamepadState {
        a: (data[4] & 0x10) != 0,
        b: (data[4] & 0x20) != 0,
        x: (data[4] & 0x40) != 0,
        y: (data[4] & 0x80) != 0,
        start: (data[4] & 0x08) != 0,
        select: (data[4] & 0x04) != 0,

        up: (data[5] & 0x01) != 0,
        down: (data[5] & 0x02) != 0,
        left: (data[5] & 0x04) != 0,
        right: (data[5] & 0x08) != 0,
        lb: (data[5] & 0x10) != 0,
        rb: (data[5] & 0x20) != 0,
        lstick: (data[5] & 0x40) != 0,
        rstick: (data[5] & 0x80) != 0,

        lt,
        rt,
        lstick_x,
        lstick_y,
        rstick_x,
        rstick_y,
    }
}

fn main() -> anyhow::Result<()> {
    let vid = 0x045e;
    let pid = 0x02ea;

    let controller_device = rusb::devices()?
        .iter()
        .find(|d| {
            if let Ok(desc) = d.device_descriptor() {
                desc.vendor_id() == vid && desc.product_id() == pid
            } else {
                false
            }
        })
        .ok_or_else(|| anyhow!("No Xbox Controller found!"))?;

    let device_desc = controller_device.device_descriptor()?;

    let config_desc = (0..device_desc.num_configurations())
        .filter_map(|i| controller_device.config_descriptor(i).ok())
        .find(|c| c.number() == 1)
        .ok_or_else(|| anyhow!("Could not find configuration 1"))?;

    let interface = config_desc
        .interfaces()
        .find(|i| i.number() == 0x00)
        .ok_or(anyhow!("Could not find interface 0"))?;

    let interface_desc = interface
        .descriptors()
        .find(|id| id.setting_number() == 0x00)
        .ok_or_else(|| anyhow!("Could not find alternate setting 0"))?;

    let endpoint_in = interface_desc
        .endpoint_descriptors()
        .find(|e| e.direction() == rusb::Direction::In && e.number() == 0x02)
        .ok_or_else(|| anyhow!("Could not find IN endpoint 0x02"))?
        .address();

    let endpoint_out = interface_desc
        .endpoint_descriptors()
        .find(|e| e.direction() == rusb::Direction::Out && e.number() == 0x02)
        .ok_or_else(|| anyhow!("Could not find OUT endpoint 0x02"))?
        .address();

    let interface_num = interface.number();

    // Open device
    let controller_handle = controller_device.open()?;

    if controller_handle.active_configuration()? != 1 {
        controller_handle.set_active_configuration(1)?;
    }

    if controller_handle.kernel_driver_active(interface_num)? {
        controller_handle.detach_kernel_driver(interface_num)?;
    }

    controller_handle.set_auto_detach_kernel_driver(true)?;

    controller_handle.claim_interface(interface_num)?;

    // Set up the device (https://github.com/quantus/xbox-one-controller-protocol)
    controller_handle.write_interrupt(
        endpoint_out,
        &[0x05, 0x20, 0x00, 0x01, 0x00],
        Duration::from_secs(1),
    )?;

    let mut buf = [0u8; 64];

    run_game_loop(|| {
        match controller_handle.read_interrupt(endpoint_in, &mut buf, Duration::from_millis(50)) {
            Ok(bytes_read) => Some(parse_xbox_controller_data(&buf[0..bytes_read])),
            _ => None,
        }
    })?;

    Ok(())
}
