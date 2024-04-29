mod io;
use std::time::Instant;

use rscpi::usbtmc::*;
use rscpi::*;

const VID_PID: &str = "2A8D:8d01";

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

    /*let inetrface_alt_settings: Vec<InterfaceAltSetting> =
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
    println!("Endpoint out Address is: 0x{:x}", address_out);*/
}

#[test]
fn idn() {
    let mut usbtmc = open_device(VID_PID).unwrap();

    //write(&mut usbtmc, "*RST").unwrap();

    let idn = query(&mut usbtmc, "*IDN?").unwrap();
    println!("{}", idn);
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

    write(&mut usbtmc, "ACQuire:POINts:ANALog 200e6").unwrap();
    check_scpi_error(&mut usbtmc);

    //write(&mut usbtmc, ":WAVeform:TYPE RAW").unwrap();
    //check_scpi_error(&mut usbtmc);

    write(&mut usbtmc, ":CHANnel1:DISPlay ON").unwrap();

    write(&mut usbtmc, ":DIGitize").unwrap();

    write(&mut usbtmc, ":WAVeform:SOURce CHAN1").unwrap();
    check_scpi_error(&mut usbtmc);

    write(&mut usbtmc, ":WAVeform:FORMat BYTE").unwrap();
    check_scpi_error(&mut usbtmc);

    write(&mut usbtmc, ":WAVeform:STReaming ON").unwrap();
    check_scpi_error(&mut usbtmc);

    let start = Instant::now();

    let data_raw = query_raw(&mut usbtmc, ":WAVeform:DATA?").unwrap();

    println!("Capture duration: {:?}", start.elapsed());

    let data = get_data_from_raw(&data_raw).unwrap();

    io::write_to_file(data, "./output/data.bin").expect("failed to write to file");
}

fn generate_ramp_f32(num_samples: usize) -> Vec<f32> {
    let mut samples = Vec::with_capacity(num_samples);

    for n in 0..num_samples {
        // Calculate the sine value
        let sine_value = 2.0 * (n as f32 / num_samples as f32 - 0.5);

        samples.push(sine_value);
    }

    samples
}

fn generate_ramp_i16(num_samples: usize) -> Vec<i16> {
    let mut samples = Vec::with_capacity(num_samples);

    for n in 0..num_samples {
        // Calculate the sine value
        let sine_value = 2.0 * (n as f32 / num_samples as f32 - 0.5) * i16::MAX as f32;

        samples.push(sine_value as i16);
    }

    samples
}

#[test]
fn awg_screenshot() {
    let mut usbtmc = open_device(VID_PID).unwrap();

    let idn = query(&mut usbtmc, "*IDN?").unwrap();
    println!("{}", idn);

    let data_raw = query_raw(&mut usbtmc, "HCOPy:SDUMp:DATA?").unwrap();

    let data = get_data_from_raw(&data_raw).unwrap();

    io::write_to_file(&data, "./output/awg_screenshot.bmp").expect("failed to write to file");
}

#[test]
fn awg_capture() {
    let mut usbtmc = open_device(VID_PID).unwrap();

    let idn = query(&mut usbtmc, "*IDN?").unwrap();
    println!("{}", idn);

    write(&mut usbtmc, "*CLS").unwrap();

    //write(&mut usbtmc, "*RST").unwrap();

    let data = generate_ramp_f32(1024 * 4 as usize);

    let bytes: Vec<u8> = data.iter().flat_map(|&f| f.to_be_bytes()).collect();

    let length1 = bytes.len().to_string();
    let length2 = length1.len().to_string();

    let prefix = format!("DATA:ARB myArb1,#{}{}", length2, length1);

    println!("Prefix =  {}", prefix);

    let mut result = prefix.into_bytes();

    result.extend_from_slice(&bytes);
    result.extend_from_slice(b"\n");

    println!("Length = {}", result.len());

    let _ = send_command_raw_binary(&mut usbtmc, &result, false).unwrap();

    check_scpi_error(&mut usbtmc);

    write(&mut usbtmc, "FUNCtion:ARBitrary \"myArb1\"").unwrap();
    check_scpi_error(&mut usbtmc);

    write(&mut usbtmc, "FUNCtion ARB").unwrap();
    check_scpi_error(&mut usbtmc);
}

#[test]
pub fn awg_file() {
    let mut usbtmc = open_device(VID_PID).unwrap();

    let idn = query(&mut usbtmc, "*IDN?").unwrap();
    println!("{}", idn);

    write(&mut usbtmc, "*CLS").unwrap();

    write(&mut usbtmc, "MMEMory:DOWNload:FNAMe \"USB:\\file1.arb\"").unwrap();
    check_scpi_error(&mut usbtmc);

    //let data = generate_ramp(1024 * 16 as usize);

    let data: Vec<i16> = generate_ramp_i16(10770);

    let header = format!(
        "File Format:1.10
Checksum:16361
Channel Count:1
Sample Rate:40000.000000
High Level:1.000000
Low Level:-1.000000
Marker Point:5000
Data Type:\"short\"
Filter:\"step\"
Data Points:{}
Data:\n",
        data.len()
    );

    //concat the header and data to byte array
    let mut bytes = header.into_bytes();

    for i in 0..data.len() {
        let line = format!("{}\n", data[i]);
        bytes.extend_from_slice(line.as_bytes());
    }

    let length1 = bytes.len().to_string();
    let length2 = length1.len().to_string();

    let prefix = format!("MMEMory:DOWNload:DATA #{}{}", length2, length1);

    println!("Prefix =  {}", prefix);

    let mut result = prefix.into_bytes();

    result.extend_from_slice(&bytes);
    result.extend_from_slice(b"\n");

    println!("Length = {}", result.len());

    let _ = write_binary(&mut usbtmc, &result).unwrap();
    check_scpi_error(&mut usbtmc);

    // write file to local storage
    io::write_to_file(&bytes, "./output/data.arb").expect("failed to write to file");
}
