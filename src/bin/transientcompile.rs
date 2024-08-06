//! Compiler that transforms Transient-C into TransientIR. (TIR)
//! Currently under development.




use std::env::args;
use std::io::{Read, Write};
use std::fs::File;

fn main() {
    // Verify input parameters
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        println!("Stop: Incorrect amount of arguments!");
        return;
    }

    // Open file for reading
    let mut input_file = match File::open(&args[1]) {
        Ok(x) => x,
        Err(_) => {
            panic!("Stop: Failed to open file");
        }
    };

    // Read bytes into buffer
    let mut transient_image: Vec<u8> = vec![];
    if let Err(_) = input_file.read_to_end(&mut transient_image) {
        panic!("Stop: Failed to read file contents");
    }
    println!("Info: File read");

}
