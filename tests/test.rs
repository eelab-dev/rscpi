mod io;
use std::time::Instant;

use rscpi::*;

const VID_PID: &str = "2A8D:0397";
const BUFF_SIZE: usize = 1024 * 1024;

#[test]
fn info() {
    let device_all = nusb::list_devices().unwrap();
    println!("{:#?}", device_all.collect::<Vec<_>>()); // Collect the iterator into a Vec and print it with pretty formatting
}

#[test]
fn config() {
    let usbtmc = open_device(VID_PID, 1024).unwrap();

    let device = &usbtmc.device;

    let config: nusb::descriptors::Configuration<'_> = device.active_configuration().unwrap();

    println!("Active configuration: {:#?}", config);
}

#[test]
fn screenshot() {
    let mut usbtmc = open_device(VID_PID, BUFF_SIZE).unwrap();

    let idn = query(&mut usbtmc, "*IDN?").unwrap();
    print!("{}", idn);

    let data = query_binary_data(&mut usbtmc, ":DISP:DATA? PNG").unwrap();
    io::write_to_file(&data, "./output/output.png").expect("failed to write to file");
}

#[test]
fn capture() {
    let message: String = String::from("Hello fellow Rustaceans!");

    println!("{}", message);

    let mut usbtmc = open_device(VID_PID, BUFF_SIZE).unwrap();

    let idn = query(&mut usbtmc, "*IDN?").unwrap();
    print!("{}", idn);

    println!("{}", query(&mut usbtmc, ":CHANnel1:SCALe?").unwrap());

    write(&mut usbtmc, ":TIMebase:MODE MAIN").unwrap();

    write(&mut usbtmc, ":WAVeform:POINts:MODE RAW").unwrap();

    write(&mut usbtmc, ":DIGitize CHANnel1").unwrap();

    write(&mut usbtmc, ":WAVeform:FORMat BYTE").unwrap();

    //write(&mut usbtmc, ":WAVeform:POINts 10151").unwrap();

    let start = Instant::now();

    let data_raw = query_raw(&mut usbtmc, ":WAVeform:DATA?").unwrap();

    println!("Capture duration: {:?}", start.elapsed());

    let data = get_data_from_raw(&data_raw).unwrap();

    io::write_to_file(data, "./output/data.bin").expect("failed to write to file");
}
