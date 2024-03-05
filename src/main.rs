/* https://github.com/rogerioadris/rust-usbtmc/blob/main/src/instrument.rs
*
*/

use byteorder::{ByteOrder, LittleEndian};
use futures_lite::future::block_on;
use nusb::{self, transfer::RequestBuffer};

const USBTMC_MSGID_DEV_DEP_MSG_OUT: u8 = 1;
const USBTMC_MSGID_DEV_DEP_MSG_IN: u8 = 2;

fn pack_bulk_out_header(msgid: u8) -> Vec<u8> {
    let last_btag: u8 = 0x00;
    let btag: u8 = (last_btag % 255) + 1;
    //last_btag = btag;

    // BBBx
    vec![msgid, btag, !btag & 0xFF, 0x00]
}

fn little_write_u32(size: u32, len: u8) -> Vec<u8> {
    let mut buf = vec![0; len as usize];
    LittleEndian::write_u32(&mut buf, size);

    buf
}

fn pack_dev_dep_msg_out_header(transfer_size: usize, eom: bool) -> Vec<u8> {
    let mut hdr = pack_bulk_out_header(USBTMC_MSGID_DEV_DEP_MSG_OUT);

    hdr.append(&mut little_write_u32(transfer_size as u32, 4));
    hdr.push(if eom { 0x01 } else { 0x00 });
    hdr.append(&mut vec![0x00; 3]);

    hdr
}

fn pack_dev_dep_msg_in_header(transfer_size: usize, term_char: u8) -> Vec<u8> {
    let mut hdr = pack_bulk_out_header(USBTMC_MSGID_DEV_DEP_MSG_IN);

    hdr.append(&mut little_write_u32(transfer_size as u32, 4));
    hdr.push(if term_char == 0 { 0x00 } else { 0x02 });
    hdr.push(term_char);
    hdr.append(&mut vec![0x00; 2]);

    hdr
}

fn is_command(data: &[u8]) -> bool {
    // Define the pattern you are looking for
    let pattern = b"?\n";

    // Use the ends_with method to check if the data ends with the pattern
    data.ends_with(pattern)
}

fn main() {
    //let last_btag: u8 = 0x00;

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

    let config = device.active_configuration().unwrap();

    println!("Active configuration: {:#?}", config);

    /* Here is the SCPI command should end with \n */
    /**************************************************/
    let data = b"*IDN?\n";
    /**************************************************/

    let offset: usize = 0;
    let num: usize = data.len();

    let block = &data[offset..(num - offset)];

    let eom = false;
    let size: usize = block.len();

    let mut req = pack_dev_dep_msg_out_header(size, eom);
    let mut b: Vec<u8> = block.iter().cloned().collect();
    req.append(&mut b);
    req.append(&mut vec![0x00; (4 - (size % 4)) % 4]);

    println!("Sending command");

    let ok = block_on(interface.bulk_out(0x02, req))
        .into_result()
        .unwrap();

    println!("ok: {:?}", ok);

    let command = is_command(data);

    if command {
        println!("Command detected");

        let max_transfer_size: usize = 1024 * 1024;

        let send = pack_dev_dep_msg_in_header(max_transfer_size, 0);
        let ok2 = block_on(interface.bulk_out(0x02, send))
            .into_result()
            .unwrap();

        println!("ok2: {:?}", ok2);

        //let buffer = [0; 512];
        let request_buffer = RequestBuffer::new(1024);
        let okr = block_on(interface.bulk_in(0x81, request_buffer))
            .into_result()
            .unwrap();

        println!("okr: {:?}", okr);

        // Convert the numeric values to a String
        let ascii_string: String = okr.iter().map(|&c| c as char).collect();

        // Print the ASCII string to the terminal
        println!("ASCII String: {}", ascii_string);
    } else {
        println!("No command detected. exiting.");
    }
}
