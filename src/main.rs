use async_std::future::join;
use async_std::io;
use async_std::prelude::*;
use async_std::stream;
use async_std::task;
use async_std::fs;
use async_std::os::unix::io::AsRawFd;

use futures::{channel, FutureExt, SinkExt};

use termios::{self, Termios};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

type Sender<T> = channel::mpsc::UnboundedSender<T>;
type Receiver<T> = channel::mpsc::UnboundedReceiver<T>;

type BSender<T> = channel::mpsc::Sender<T>;
type BReceiver<T> = channel::mpsc::Receiver<T>;

#[derive(Debug)]
enum Packet {
    Inbound(String),
    Char(char), // TODO: Create Key enum
    Ignore,
    Done,
}

async fn input_loop(shutdown: BReceiver<Packet>, mut wire: Sender<Packet>) -> Result<()> {
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
            // eprintln!("[input_loop.std_lines] New line: {:?}", res);
            let line = res.expect("Error reading from STDIN");

            Packet::Inbound(line)
        });
        // .chain(stream::once(Packet::Done));

    // TODO: Transform sequence of bytes into Keys (arrow keys, Ctrl, chars, etc)
    let tty_chars = tty_reader.bytes()
        .map(|res| res.expect("Error reading from PTTY"))
        .scan(Vec::new(), |state, byte| {
            eprintln!("[input_loop.std_chars] New byte: {:?} {:?}", state, byte);
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
        eprintln!("[input_loop.while] got packet: {:?}", packet);

        match packet {
            // Shutting down from here works!
            // I think the problem was the back and forth between the broker
            // task and this one with the shutdown channel
            // I'll try to keep the direction of comms one way only
            Packet::Char('\n') => break,
            _ => wire.send(packet).await?,
        }
    }

    eprintln!("[input_loop] drop(wire)");
    drop(wire);

    eprintln!("[input_loop] drop(all)");
    drop(all);

    eprintln!("[input_loop] out");

    Ok(())
}

async fn broker_loop(mut packets: Receiver<Packet>, mut shutdown: BSender<Packet>) -> Result<()> {
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
        eprintln!("[broker] got packet: {:?}", packet);

        let line = match packet {
            Packet::Inbound(s) => {
                let l = format!("STDIN: {}", s);
                Some(l)
            },
            Packet::Char(ch) => {
                match ch {
                    '\n' | '\u{1b}' => {
                        // eprintln!("[broker] sending shutdown");

                        // shutdown
                        //     .try_send(Packet::Done)
                        //     .expect("Error shutting down");

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

        eprintln!("[broker] got line: {:?}", line);

        if let Some(l) = line {
            let l = format!("{}\n", l);
            tty_out.write_all(l.as_bytes()).await?;
            tty_out.flush().await?;
        }
    };

    // eprintln!("[broker] restore tty_out");
    // termios::tcsetattr(fd, termios::TCSANOW, &tty)
    //     .expect("Error restoring the original tty configuration");

    eprintln!("[broker] drop(shutdown)");
    drop(shutdown);

    eprintln!("[broker] out");

    Ok(())
}

fn main() -> Result<()> {
    let res = task::block_on(async {
        // Let's add a shutdown channel so we can move logic to other futures/tasks later
        let (shutdown_sender, shutdown_receiver) = channel::mpsc::channel::<Packet>(1);
        // let (mut output_sender, output_receiver) = channel::mpsc::unbounded::<Packet>();
        let (broker_sender, broker_receiver) = channel::mpsc::unbounded::<Packet>();

        let broker = spawn_and_log_error(broker_loop(broker_receiver, shutdown_sender));
        let input = spawn_and_log_error(input_loop(shutdown_receiver, broker_sender));

        join!(broker, input).await;

        println!("All done!");

        Ok(())
    });

    eprintln!("[main] after executor");

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

        eprintln!("[spawn_and_log_error] future done");
    })
}
