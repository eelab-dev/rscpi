use nusb;
mod usbtmc;

fn main() {
    //let last_btag: u8 = 0x00;

    let message: String = String::from("Hello fellow Rustaceans!");

    print!("{}\n", message);

    let device_all = nusb::list_devices().unwrap();
    println!("{:#?}", device_all.collect::<Vec<_>>()); // Collect the iterator into a Vec and print it with pretty formatting

    let device_info: nusb::DeviceInfo = nusb::list_devices()
        .unwrap()
        .find(|dev| dev.vendor_id() == 0x2A8D && dev.product_id() == 0x0397)
        .expect("device not connected");

    let device: nusb::Device = device_info.open().expect("failed to open device");
    let mut interface: nusb::Interface = device.detach_and_claim_interface(0).unwrap();

    let config: nusb::descriptors::Configuration<'_> = device.active_configuration().unwrap();

    println!("Active configuration: {:#?}", config);

    usbtmc::send_command(&mut interface, "*IDN?");

    usbtmc::send_command(&mut interface, ":CHANnel1:SCALe?");

    usbtmc::send_command(&mut interface, ":DIGitize CHANnel1");

    usbtmc::send_command(&mut interface, ":WAVeform:FORMat BYTE");

    usbtmc::send_command(&mut interface, ":WAVeform:POINts 1000");

    usbtmc::send_command(&mut interface, ":WAVeform:DATA?");
}
