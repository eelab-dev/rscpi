use std::fs::File;
use std::io::prelude::*;

pub fn write_to_file(data: &[u8], path: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?; // Create or overwrite a file named "output.bin"

    // Write the bytes to the file
    file.write_all(data)?;

    #[cfg(debug_assertions)]
    println!("Wrote {} bytes to file", data.len());

    Ok(())
}
