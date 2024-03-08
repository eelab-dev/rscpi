use std::fs::File;
use std::io::prelude::*;

pub fn write_to_file(data: &[u8]) -> std::io::Result<()> {
    let mut file = File::create("output.png")?; // Create or overwrite a file named "output.bin"

    // Write the bytes to the file
    file.write_all(&data)?;

    Ok(())
}
