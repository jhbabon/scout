#[macro_use]
extern crate log;

use async_std::future::join;
use async_std::io;
use async_std::prelude::*;
use async_std::task;
use async_std::fs;
use async_std::os::unix::io::AsRawFd;

use futures::{channel, SinkExt};

use termios::{self, Termios};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

type Sender<T> = channel::mpsc::UnboundedSender<T>;
type Receiver<T> = channel::mpsc::UnboundedReceiver<T>;

#[derive(Debug)]
enum Packet {
    Inbound(String),
    Char(char), // TODO: Create Key enum
    Ignore,
    Done,
}

async fn input_loop(mut wire: Sender<Packet>) -> Result<()> {
    // Get all inputs
    let stdin = io::stdin();
    let tty_in = get_tty().await?;

    // TODO: This raw mode may need to be different than tty_out
    // BEGIN: RAW MODE
    let fd = tty_in.as_raw_fd();
    let tty = Termios::from_fd(fd)?;
    let mut raw_tty = tty;
    // termios::cfmakeraw(&mut raw_tty);
    raw_tty.c_lflag &= !(termios::ICANON | termios::ECHO | termios::ECHONL | termios::IEXTEN);
    // raw_tty.c_cc[termios::VMIN] = 0;
    // raw_tty.c_cc[termios::VTIME] = 1;
    termios::tcsetattr(fd, termios::TCSANOW, &raw_tty)?;
    // END:   RAW MODE

    let std_reader = io::BufReader::new(stdin);
    let tty_reader = io::BufReader::new(tty_in);

    let std_lines = std_reader.lines()
        .map(|res| {
            let line = res.expect("Error reading from STDIN");

            Packet::Inbound(line)
        });
        // .chain(stream::once(Packet::Done));

    // TODO: Transform sequence of bytes into Keys (arrow keys, Ctrl, chars, etc)
    let tty_chars = tty_reader.bytes()
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

    // This select works in a round robin fashion
    let mut all = futures::stream::select(tty_chars, std_lines);

    while let Some(packet) = all.next().await {
        match packet {
            // Shutting down from here works!
            // I think the problem was the back and forth between the broker
            // task and this one with the shutdown channel
            // I'll try to keep the direction of comms one way only
            Packet::Char('\n') => {
                wire.send(Packet::Done).await?;
                break
            },
            _ => wire.send(packet).await?,
        }
    }

    drop(wire);
    drop(all);

    Ok(())
}

async fn broker_loop(mut packets: Receiver<Packet>) -> Result<()> {
    // Get all outputs
    let mut tty_out = get_tty().await?;

    // BEGIN: RAW MODE
    let fd = tty_out.as_raw_fd();
    let tty = Termios::from_fd(fd)?;
    let mut raw_tty = tty;
    raw_tty.c_lflag &= !(termios::ICANON | termios::ECHO);
    termios::tcsetattr(fd, termios::TCSANOW, &raw_tty)?;
    // END:   RAW MODE

    while let Some(packet) = packets.next().await {
        let line = match packet {
            Packet::Inbound(s) => {
                let l = format!("STDIN: {}", s);
                Some(l)
            },
            Packet::Char(ch) => {
                match ch {
                    '\n' | '\u{1b}' => {
                        // FIXME: This will handled before
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
    };

    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();

    debug!("[main] start");

    let res = task::block_on(async {
        // let (mut output_sender, output_receiver) = channel::mpsc::unbounded::<Packet>();
        let (broker_sender, broker_receiver) = channel::mpsc::unbounded::<Packet>();

        let broker = spawn_and_log_error(broker_loop(broker_receiver));
        let input = spawn_and_log_error(input_loop(broker_sender));

        join!(broker, input).await;

        println!("All done!");

        Ok(())
    });

    debug!("[main] end: {:?}", res);

    res
}

async fn get_tty() -> Result<fs::File> {
    let tty = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .await?;

    Ok(tty)
}

fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    task::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{}", e)
        }
    })
}
