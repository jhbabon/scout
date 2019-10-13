use async_std::io;
use async_std::prelude::*;
use async_std::task;
use async_std::fs;
use async_std::os::unix::io::AsRawFd;

use futures::{channel, FutureExt, SinkExt};

use termios::{self, Termios};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type Sender<T> = channel::mpsc::UnboundedSender<T>;
type Receiver<T> = channel::mpsc::UnboundedReceiver<T>;

enum Packet {
    Inbound(String),
    Char(char), // TODO: Create Key enum
    Ignore,
    Done,
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

        let std_lines = std_reader.lines()
            .map(|res| {
                let line = res.expect("Error reading from STDIN");

                Packet::Inbound(line)
            });

        // TODO: Transform sequence of bytes into Keys (arrow keys, Ctrl, chars, etc)
        let tty_lines = tty_reader.bytes()
            .map(|res| res.expect("Error reading from PTTY"))
            .scan(Vec::new(), |state, byte| {
                state.push(byte);

                let packet = match String::from_utf8(state.clone()) {
                    Ok(s) => {
                        state.clear();
                        if let Some(ch) = s.chars().next() {
                            Some(Packet::Char(ch))
                        } else {
                            Some(Packet::Ignore)
                        }
                    },
                    Err(_) => {
                        // probably the program needs the next byte to make sense of it
                        Some(Packet::Ignore)
                    }
                };

                packet
            });

        // Let's add a shutdown channel so we can move logic to other futures/tasks later
        let (mut shutdown_sender, shutdown_receiver) = channel::mpsc::channel::<Packet>(1);

        // This select works in a round robin fashion
        let ins = futures::stream::select(tty_lines, std_lines);
        let mut all = futures::stream::select(shutdown_receiver, ins);

        while let Some(packet) = all.next().await {
            let line = match packet {
                Packet::Inbound(s) => {
                    let l = format!("STDIN: {}", s);
                    Some(l)
                },
                Packet::Char(ch) => {
                    match ch {
                        '\n' | '\u{1b}' => {
                            shutdown_sender
                                .try_send(Packet::Done)
                                .expect("Error shutting down");

                            None
                        },
                        _  => {
                            let l = format!("TTYIN: {:?}", ch);
                            Some(l)
                        }
                    }
                },
                Packet::Ignore => None,
                Packet::Done => break,
            };

            if let Some(l) = line {
                let l = format!("{}\n", l);
                tty_out.write_all(l.as_bytes()).await?;
                tty_out.flush().await?;
            }
        }

        drop(shutdown_sender);
        println!("All done!");

        Ok(())
    })
}

async fn get_tty() -> Result<fs::File> {
    let tty = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .await?;

    Ok(tty)
}
