use nusb;

fn main() {
    let message: String = String::from("Hello fellow Rustaceans!");

    print!("{}\n", message);

    let device = nusb::list_devices().unwrap();
    println!("{:#?}", device.collect::<Vec<_>>()); // Collect the iterator into a Vec and print it with pretty formatting
}
