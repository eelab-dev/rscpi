mod usbtmc;
use crate::usbtmc::*;
use usbtmc::UsbtmcErrors;

pub struct Usbtmc {
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
