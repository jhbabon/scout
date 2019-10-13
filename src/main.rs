#![recursion_limit="256"]

use async_std::io;
use async_std::prelude::*;
use async_std::task;
use async_std::fs;
use async_std::os::unix::io::AsRawFd;

use termios::{self, Termios};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

enum Packet {
    Inbound(String),
    Byte(u8),
}

async fn get_tty() -> Result<fs::File> {
    let tty = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .await?;

    Ok(tty)
}

fn main() -> Result<()> {
    task::block_on(async {
        // Get all inputs
        let stdin = io::stdin();
        let tty_in = get_tty().await?;

        // Get all outputs
        let mut tty_out = get_tty().await?;

        // TODO: This raw mode may need to be different than tty_out
        // BEGIN: RAW MODE
        let fd = tty_in.as_raw_fd();
        let tty = Termios::from_fd(fd)?;
        let mut raw_tty = tty;
        raw_tty.c_lflag &= !(termios::ICANON | termios::ECHO);
        termios::tcsetattr(fd, termios::TCSANOW, &raw_tty)?;
        // END:   RAW MODE

        // BEGIN: RAW MODE
        let fd = tty_out.as_raw_fd();
        let tty = Termios::from_fd(fd)?;
        let mut raw_tty = tty;
        raw_tty.c_lflag &= !(termios::ICANON | termios::ECHO);
        termios::tcsetattr(fd, termios::TCSANOW, &raw_tty)?;
        // END:   RAW MODE

        let std_reader = io::BufReader::new(stdin);
        let tty_reader = io::BufReader::new(tty_in);

        let std_lines = std_reader.lines().map(|l| {
            if let Ok(line) = l {
                Some(Packet::Inbound(line))
            } else {
                None
            }
        });

        // FIXME: Probably is better if the tty stream does the byte to chars conversion
        // let tty_lines = tty_reader.bytes().scan(Vec::new(), |state, b| {
        //     if let Ok(byte) = b {
        //         state.push(byte);
        //         Some(Some(Packet::Byte(byte)))
        //     } else {
        //         None
        //     }
        // });

        let tty_lines = tty_reader.bytes().map(|b| {
            if let Ok(byte) = b {
                Some(Packet::Byte(byte))
            } else {
                None
            }
        });

        // This select works in a round robin fashion
        let mut all = futures::stream::select(tty_lines, std_lines);

        let mut buf = Vec::with_capacity(10);

        while let Some(packet) = all.next().await {
            let line = match packet {
                Some(Packet::Inbound(s)) => {
                    let l = format!("STDIN: {}", s);
                    Some(l)
                },
                Some(Packet::Byte(byte)) => {
                    // FIXME: some keys have more than one byte, i.e: Up, Down, etc
                    //   these start with escape sequences, the char: \u{1b}
                    //   In order to intepret them correctly the progam has to
                    //   wait until the next squence of bytes based on the first one
                    buf.push(byte);
                    let chars = match String::from_utf8(buf.clone()) {
                        Ok(s) => {
                            buf.clear();
                            Some(s)
                        },
                        Err(_) => {
                            // probably the program needs the next byte to make sense of it
                            None
                        }
                    };

                    if let Some(chrs) = chars {
                        let ch = chrs.chars()
                            .fold("".to_string(), |acc, key| format!("{}{:?}", acc, key));
                        Some(ch)
                    } else {
                        None
                    }

                },
                None => None,
            };

            if let Some(l) = line {
                let l = format!("{}\n", l);
                tty_out.write_all(l.as_bytes()).await?;
                tty_out.flush().await?;
            }
        }

        Ok(())
    })
}
