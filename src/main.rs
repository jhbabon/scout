use std::{fs, io};
use std::io::{Read, Write};

fn magic() -> Result<(), io::Error> {
    println!("The magic begins");

    // let mut input = String::new();
    // try!(io::stdin().read_to_string(&mut input));

    let mut tty = try!(fs::OpenOptions::new().read(true).write(true).open("/dev/tty"));

    // let _ = writeln!(&mut tty, "{}", input);
    let mut counter = 0;

    'event: loop {
        let mut buffer = String::new();
        try!(tty.read_to_string(&mut buffer));
        let _ = writeln!(&mut tty, "Your input: {}", buffer);
        counter += 1;

        if counter > 3 {
            break 'event;
        }
    };

    Ok(())
}

fn main() {
    let _ = magic();
}
