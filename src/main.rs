use futures_lite::future::block_on;
use nusb;
use nusb::transfer::{ControlOut, ControlType, Recipient};

fn main() {
    let message: String = String::from("Hello fellow Rustaceans!");

    print!("{}\n", message);

    let device_all = nusb::list_devices().unwrap();
    println!("{:#?}", device_all.collect::<Vec<_>>()); // Collect the iterator into a Vec and print it with pretty formatting

    let device_info = nusb::list_devices()
        .unwrap()
        .find(|dev| dev.vendor_id() == 0x2A8D && dev.product_id() == 0x0397)
        .expect("device not connected");

    let device = device_info.open().expect("failed to open device");
    let interface = device.detach_and_claim_interface(0).unwrap();

    let command = b"*IDN?\n";
    //let mut buffer = [0; 256];

    println!("Sending command");

    let ok = block_on(interface.control_out(ControlOut {
        control_type: ControlType::Vendor,
        recipient: Recipient::Device,
        request: 0x43,
        value: 0x0,
        index: 0x0,
        data: command,
    }))
    .into_result()
    .unwrap();

    println!("ok: {:?}", ok);
} // Add this closing curly brace
