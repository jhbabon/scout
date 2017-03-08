use std::{fs, io};
use std::io::{Read, Write};

fn magic() -> Result<(), io::Error> {
    println!("The magic begins");

    let mut input = String::new();
    try!(io::stdin().read_to_string(&mut input));

    let mut tty = try!(fs::OpenOptions::new().read(true).write(true).open("/dev/tty"));

    let _ = writeln!(&mut tty, "{}", input);

    let mut buffer = String::new();
    match tty.read_to_string(&mut buffer) {
        Ok(_) => println!("Input: {}", buffer),
        Err(e) => println!("Error: {}", e),
    };

    Ok(())
}

fn main() {
    let _ = magic();
}
