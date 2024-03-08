/* https://github.com/rogerioadris/rust-usbtmc/blob/main/src/instrument.rs
*
*/

use byteorder::{ByteOrder, LittleEndian};
use futures_lite::future::block_on;
use nusb::transfer::RequestBuffer;

const USBTMC_MSGID_DEV_DEP_MSG_OUT: u8 = 1;
const USBTMC_MSGID_DEV_DEP_MSG_IN: u8 = 2;

fn print_array(bytes: Vec<u8>) {
    let len = bytes.len();
    if len <= 25 {
        for byte in &bytes {
            print!("{} ", byte);
        }
        println!(); // Print a newline for neatness
    } else {
        print!("[");
        for byte in bytes.iter().take(20) {
            print!("{} ", byte);
        }
        print!(" ... ");
        for byte in bytes.iter().skip(len - 5) {
            print!("{} ", byte);
        }
        println!("]");
    }
}

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
    // Define the byte you are looking for
    let question_mark = b'?';

    // Use the contains method to check if the data contains the byte
    data.contains(&question_mark)
}

fn read_data_transfer(
    interface: &mut nusb::Interface,
    buffer_size: usize,
    big_buffer: &mut Vec<u8>,
) -> usize {
    let request_buffer = RequestBuffer::new(buffer_size);
    let okr = block_on(interface.bulk_in(0x81, request_buffer))
        .into_result()
        .unwrap();

    print!("okr->: ");
    print_array(okr.to_vec());

    big_buffer.extend_from_slice(&okr);

    let usb_packet_size = okr.len();
    println!("usb packet size: {}", usb_packet_size);

    usb_packet_size
}

fn read_data(
    interface: &mut nusb::Interface,
    big_big_buffer: &mut Vec<u8>,
    req_buffer_size: Option<usize>,
) -> bool {
    let max_transfer_size: usize = 1024 * 1024;
    let buffer_size: usize;
    let mut big_buffer: Vec<u8> = Vec::new();

    match req_buffer_size {
        Some(size) => {
            if size > max_transfer_size {
                panic!("Requested buffer size is greater than max transfer size");
            }
            buffer_size = req_buffer_size.unwrap();
        }
        None => {
            buffer_size = 1024 as usize;
        }
    }

    let send = pack_dev_dep_msg_in_header(max_transfer_size, 0);
    let ok2 = block_on(interface.bulk_out(0x02, send))
        .into_result()
        .unwrap();

    println!("ok2->: {:?}", ok2);

    let mut usb_packet_recv_size = read_data_transfer(interface, buffer_size, &mut big_buffer);

    // Separate the bytes
    let (header, _): (&[u8], &[u8]) = big_buffer.split_at(12);

    // Convert the first part to hexadecimal values and print
    print!("Header Hex values: ");
    for byte in header {
        print!("{:02x} ", byte);
    }
    println!();

    let payload_size = LittleEndian::read_u32(&header[4..8]) as usize;
    println!("Payload size in header: {}", payload_size);

    let eom: bool = header[8] == 0x01;
    println!("EOM: {}", eom);

    while usb_packet_recv_size == buffer_size {
        println!("Packet size equals buffer size. Reading more data.");
        usb_packet_recv_size = read_data_transfer(interface, buffer_size, &mut big_buffer);
    }

    big_big_buffer.extend_from_slice(&big_buffer[12..]);

    eom
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

    println!("Sending command: {:?}", command);

    let ok: nusb::transfer::ResponseBuffer = block_on(interface.bulk_out(0x02, req))
        .into_result()
        .unwrap();

    println!("ok->: {:?}", ok);

    let query = is_query(data);

    let mut big_big_buffer: Vec<u8> = Vec::new();

    if query {
        println!("query detected");
        let mut eom = read_data(interface, &mut big_big_buffer, None);

        while !eom {
            println!("eom is false. Reading more data.");
            eom = read_data(interface, &mut big_big_buffer, None);
        }
        println!(
            "transfer complete. total payload size: {}",
            big_big_buffer.len()
        );
        let term_recv: bool = big_big_buffer[big_big_buffer.len() - 1] == 0x0A; //=10
        println!("term_recv: {}", term_recv);

        if big_big_buffer.len() > 100 {
            println!("The first 10 bytes of the payload:");
            let ten_bytes = &big_big_buffer[0..10];
            let ascii_string: String = ten_bytes.iter().map(|&b| b as char).collect();
            println!("{}", ascii_string);
        }
    } else {
        println!("No command detected. exiting.");
    }
    println!("");
}
