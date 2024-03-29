mod usbtmc;
use crate::usbtmc::*;
use usbtmc::UsbtmcErrors;

use nusb::descriptors::InterfaceAltSetting;
use nusb::transfer::Direction;

pub struct Usbtmc {
    pub device: nusb::Device,
    pub interface: nusb::Interface,
    endpoint_in_addr: u8,
    endpoint_out_addr: u8,
    endpoint_in_max_packet_size: usize,
    #[allow(dead_code)]
    endpoint_out_max_packet_size: usize,
}

pub fn query(usbtmc: &mut Usbtmc, command: &str) -> Result<String, UsbtmcErrors> {
    send_command(usbtmc, command)
}

pub fn query_raw(usbtmc: &mut Usbtmc, command: &str) -> Result<Vec<u8>, UsbtmcErrors> {
    send_command_raw(usbtmc, command)
}

pub fn write(usbtmc: &mut Usbtmc, command: &str) -> Result<(), UsbtmcErrors> {
    let _ = send_command_raw(usbtmc, command)?;

    Ok(())
}

pub fn open_device(vid_pid: &str) -> Result<Usbtmc, UsbtmcErrors> {
    let vid = u16::from_str_radix(&vid_pid[0..4], 16).unwrap();
    let pid = u16::from_str_radix(&vid_pid[5..9], 16).unwrap();

    let device_info: nusb::DeviceInfo = nusb::list_devices()
        .unwrap()
        .find(|dev| dev.vendor_id() == vid && dev.product_id() == pid)
        .expect("device not connected");

    let device: nusb::Device = device_info.open().expect("failed to open device");
    let interface: nusb::Interface = device
        .detach_and_claim_interface(0)
        .expect("failed to claim interface");

    let config: nusb::descriptors::Configuration<'_> = device
        .active_configuration()
        .expect("failed to get active configuration");

    let inetrface_alt_settings: Vec<InterfaceAltSetting> =
        config.interface_alt_settings().collect();

    let endpoint_in = inetrface_alt_settings[0]
        .endpoints()
        .find(|ep| ep.direction() == Direction::In)
        .expect("failed to find endpoint_in");

    let address_in = endpoint_in.address();

    let endpoint_out = inetrface_alt_settings[0]
        .endpoints()
        .find(|ep| ep.direction() == Direction::Out)
        .expect("failed to find endpoint_out");

    let address_out = endpoint_out.address();

    let endpoint_in_max_packet_size = endpoint_in.max_packet_size();
    let endpoint_out_max_packet_size = endpoint_out.max_packet_size();

    Ok(Usbtmc {
        device,
        interface,
        endpoint_in_addr: address_in,
        endpoint_out_addr: address_out,
        endpoint_in_max_packet_size,
        endpoint_out_max_packet_size,
    })
}

pub fn get_data_from_raw(raw_data: &[u8]) -> Result<&[u8], UsbtmcErrors> {
    if raw_data[0] == b'#' {
        let num_bytes = String::from_utf8(raw_data[1..2].to_vec())
            .unwrap()
            .parse::<usize>()
            .unwrap();

        let data_size_ascii = String::from_utf8(raw_data[2..(2 + num_bytes)].to_vec()).unwrap();
        let data_size = data_size_ascii.parse::<usize>().unwrap();

        if data_size != raw_data.len() - (2 + num_bytes) {
            return Err(UsbtmcErrors::InvalidData);
        }

        let data = &raw_data[(2 + num_bytes)..];

        Ok(data)
    } else {
        Err(UsbtmcErrors::InvalidData)
    }
}

pub fn query_binary_data(usbtmc: &mut Usbtmc, command: &str) -> Result<Vec<u8>, UsbtmcErrors> {
    let data_raw = query_raw(usbtmc, command)?;

    let data = get_data_from_raw(&data_raw)?;

    Ok(data.to_vec())
}
