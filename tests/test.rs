mod io;
use std::time::Instant;

use nusb::descriptors::InterfaceAltSetting;
use nusb::transfer::Direction;
use rscpi::*;

const VID_PID: &str = "2A8D:0397";

#[test]
fn info() {
    let device_all = nusb::list_devices().unwrap();
    println!("{:#?}", device_all.collect::<Vec<_>>()); // Collect the iterator into a Vec and print it with pretty formatting
}

#[test]
fn config() {
    let usbtmc = open_device(VID_PID).unwrap();

    let device = &usbtmc.device;

    let config: nusb::descriptors::Configuration<'_> = device.active_configuration().unwrap();

    println!("Active configuration: {:#?}", config);

    let inetrface_alt_settings: Vec<InterfaceAltSetting> =
        config.interface_alt_settings().collect();

    // find the address of endpoint which matches Bulk IN
    let endpoint_in = inetrface_alt_settings[0]
        .endpoints()
        .find(|ep| ep.direction() == Direction::In)
        .unwrap();

    // find the property address of endpoint_in
    let address_in = endpoint_in.address();

    // print the address in hex
    println!("Endpoint in Address is: 0x{:x}", address_in);

    // find the address of endpoint which matches Bulk OUT
    let endpoint_out = inetrface_alt_settings[0]
        .endpoints()
        .find(|ep| ep.direction() == Direction::Out)
        .unwrap();

    // find the property address of endpoint_out
    let address_out = endpoint_out.address();

    // print the address in hex
    println!("Endpoint out Address is: 0x{:x}", address_out);
}

#[test]
fn screenshot() {
    let mut usbtmc = open_device(VID_PID).unwrap();

    let idn = query(&mut usbtmc, "*IDN?").unwrap();
    print!("{}", idn);

    let data = query_binary_data(&mut usbtmc, ":DISP:DATA? PNG").unwrap();
    io::write_to_file(&data, "./output/screenshot.png").expect("failed to write to file");
}

fn check_scpi_error(usbtmc: &mut Usbtmc) {
    let error = query(usbtmc, ":SYSTem:ERRor?").unwrap();

    // check if the error string contains a comma
    if error.contains(',') {
        let error_code = error.split(',').nth(0).unwrap().parse::<i32>().unwrap();

        if error_code != 0 {
            let error_message = error.split(',').nth(1).unwrap();
            println!("SCPI error: {} - {}", error_code, error_message);
        }
    } else {
        let error_code = error.parse::<i32>().unwrap();

        if error_code != 0 {
            println!("SCPI error: {}", error_code);
        }
    }
}

#[test]
fn capture() {
    let message: String = String::from("Hello fellow Rustaceans!");

    println!("{}", message);

    let mut usbtmc = open_device(VID_PID).unwrap();

    let idn = query(&mut usbtmc, "*IDN?").unwrap();
    println!("{}", idn);

    write(&mut usbtmc, "*CLS").unwrap();

    write(&mut usbtmc, ":WAVeform:POINts:MODE RAW").unwrap();
    check_scpi_error(&mut usbtmc);

    write(&mut usbtmc, ":DIGitize CHANnel1, CHANnel2").unwrap();

    write(&mut usbtmc, ":WAVeform:FORMat BYTE").unwrap();
    check_scpi_error(&mut usbtmc);

    write(&mut usbtmc, ":WAVeform:SOURce CHAN1").unwrap();
    check_scpi_error(&mut usbtmc);

    let start = Instant::now();

    let data_raw = query_raw(&mut usbtmc, ":WAVeform:DATA?").unwrap();

    println!("Capture duration: {:?}", start.elapsed());

    let data = get_data_from_raw(&data_raw).unwrap();

    io::write_to_file(data, "./output/data.bin").expect("failed to write to file");
}
