mod usbtmc;
use crate::usbtmc::*;
use usbtmc::UsbtmcErrors;

pub struct Usbtmc {
    pub device: nusb::Device,
    pub interface: nusb::Interface,
    pub recv_buffer_size: usize,
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

pub fn open_device(vid_pid: &str, buff_size: usize) -> Result<Usbtmc, UsbtmcErrors> {
    let vid = u16::from_str_radix(&vid_pid[0..4], 16).unwrap();
    let pid = u16::from_str_radix(&vid_pid[5..9], 16).unwrap();

    let device_info: nusb::DeviceInfo = nusb::list_devices()
        .unwrap()
        .find(|dev| dev.vendor_id() == vid && dev.product_id() == pid)
        .expect("device not connected");

    let device: nusb::Device = device_info.open().expect("failed to open device");
    let interface: nusb::Interface = device.detach_and_claim_interface(0).unwrap();

    Ok(Usbtmc {
        device,
        interface,
        recv_buffer_size: buff_size,
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
