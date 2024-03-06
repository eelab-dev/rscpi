/* https://github.com/rogerioadris/rust-usbtmc/blob/main/src/instrument.rs
*
*/

use byteorder::{ByteOrder, LittleEndian};
use futures_lite::future::block_on;
use nusb::transfer::RequestBuffer;

const USBTMC_MSGID_DEV_DEP_MSG_OUT: u8 = 1;
const USBTMC_MSGID_DEV_DEP_MSG_IN: u8 = 2;

fn little_write_u32(size: u32, len: u8) -> Vec<u8> {
    let mut buf = vec![0; len as usize];
    LittleEndian::write_u32(&mut buf, size);

    buf
}

/*
* USBTMC document Table 1
*/
fn pack_bulk_out_header(msgid: u8) -> Vec<u8> {
    let last_btag: u8 = 0x00;
    let btag: u8 = (last_btag % 255) + 1;
    //last_btag = btag;

    // BBBx
    vec![msgid, btag, !btag & 0xFF, 0x00]
}

/*
* USBTMC document Table 3 and USB488 document Table 3
*/
fn pack_dev_dep_msg_out_header(transfer_size: usize, eom: bool) -> Vec<u8> {
    let mut header = pack_bulk_out_header(USBTMC_MSGID_DEV_DEP_MSG_OUT);

    let mut total_transfer_size: Vec<u8> = little_write_u32(transfer_size as u32, 4);
    let bm_transfer_attributes: u8 = if eom { 0x01 } else { 0x00 };

    header.append(&mut total_transfer_size);
    header.push(bm_transfer_attributes);
    header.append(&mut vec![0x00; 3]);

    // check if transfer size is not 0

    header
}

/*
* USBTMC document Table 4
*/
fn pack_dev_dep_msg_in_header(transfer_size: usize, term_char: u8) -> Vec<u8> {
    let mut header = pack_bulk_out_header(USBTMC_MSGID_DEV_DEP_MSG_IN);

    let mut max_transfer_size: Vec<u8> = little_write_u32(transfer_size as u32, 4);
    let bm_transfer_attributes: u8 = if term_char == 0 { 0x00 } else { 0x02 };

    header.append(&mut max_transfer_size);
    header.push(bm_transfer_attributes);
    header.push(term_char);
    header.append(&mut vec![0x00; 2]);

    header
}

fn is_query(data: &[u8]) -> bool {
    // Define the pattern you are looking for
    let pattern = b"?\n";

    // Use the ends_with method to check if the data ends with the pattern
    data.ends_with(pattern)
}

pub fn send_command(interface: &mut nusb::Interface, command: &str) {
    let command_with_newline = command.to_owned() + "\n";
    let data = command_with_newline.as_bytes();

    let offset: usize = 0;
    let num: usize = data.len();

    let block = &data[offset..(num - offset)];

    let eom = true;
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

    let query = is_query(data);

    if query {
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
        println!();

        // Convert the numeric values to a String
        //let ascii_string: String = okr.iter().map(|&c| c as char).collect();

        // Separate the bytes
        let (header, payload) = okr.split_at(12);

        // Convert the first part to hexadecimal values and print
        print!("Hex values: ");
        for byte in header {
            print!("{:02x} ", byte);
        }
        println!(); // Add a new line for clarity

        let payload_size = LittleEndian::read_u32(&header[4..8]) as usize;
        println!("Payload size: {}", payload_size);

        let eom = (header[8] & 0x01) != 0;
        println!("EOM: {}", eom);

        // Convert the second part to ASCII and print
        print!("ASCII values: ");
        let ascii_string = String::from_utf8(payload.to_vec()).unwrap();
        println!("{}", ascii_string);
    } else {
        println!("No command detected. exiting.");
    }
}
