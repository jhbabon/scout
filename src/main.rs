#[macro_use]
extern crate log;

use rayon::prelude::*;

use std::pin::Pin;
use std::collections::VecDeque;

use async_std::prelude::*;
use async_std::future::join;
use async_std::io;
use async_std::task;
use async_std::task::{Context, Poll};
use async_std::fs;
use async_std::os::unix::io::AsRawFd;

use futures::{channel, SinkExt};

use termios::{self, Termios};
use termion::input::TermRead;
use termion::event::Key;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

type Sender<T> = channel::mpsc::UnboundedSender<T>;
type Receiver<T> = channel::mpsc::UnboundedReceiver<T>;

#[derive(Debug)]
enum Packet {
    Inbound(String),
    // Ignore,
    Done,
    Key(Key),
}

#[derive(Debug,Clone, Default)]
struct State {
    pub query: Vec<char>,
    pub pool: Vec<String>,
    pub matches: Vec<String>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_char(&mut self, ch: char) {
        self.query.push(ch);
    }

    pub fn add_string(&mut self, string: String) {
        self.pool.push(string);
    }

    // NOTE: This is just temporary, the search should
    // be outside the state
    pub fn search(&mut self) {
        let q = self.query.iter().collect::<String>();

        self.matches = self.pool
            .par_iter()
            .filter(|s| s.contains(q.as_str()))
            .map(|s| s.clone())
            .collect();
    }
}

fn inbound(
    r: impl io::Read + Unpin,
) -> impl Stream<Item = Packet> + Unpin {
    struct Inbound<R> {
        reader: R,
        buffer: VecDeque<Key>,
    };

    impl<R: io::Read + Unpin> Inbound<R> {
        fn new(r: R) -> Self {
            Self {
                reader: r,
                buffer: VecDeque::new(),
            }
        }
    }

    impl<R: io::Read + Unpin> Stream for Inbound<R> {
        type Item = Packet;

        fn poll_next(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Option<Self::Item>> {

            if let Some(k) = self.buffer.pop_front() {
                return Poll::Ready(Some(Packet::Key(k)));
            }

            let mut buf = vec![0; 4];
            let mut fut = self.reader.read(&mut buf);

            match Pin::new(&mut fut).poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Ok(n)) => {
                    debug!("[inbound.poll_next()] bytes read: {:?}", n);

                    let tmp: Vec<u8> = buf
                        .iter()
                        .take(n)
                        .cloned()
                        .collect();

                    let mut keys = tmp
                        .keys()
                        .filter(|k| k.is_ok())
                        .map(|k| k.unwrap());

                    let key = keys.next();
                    debug!("[inbound.poll_next()] key: {:?}", key);

                    while let Some(k) = keys.next() {
                        match k {
                            Key::Null => (),
                            _ => self.buffer.push_back(k),
                        }
                    }
                    debug!("[inbound.poll_next()] extra keys: {:?}", self.buffer);

                    Poll::Ready(key.map(|k| Packet::Key(k)))
                },
                Poll::Ready(Err(_)) => {
                    Poll::Ready(None)
                },
            }
        }
    }

    Inbound::new(r)
}

async fn input_loop(mut wire: Sender<Packet>) -> Result<()> {
    debug!("[input_loop] start");

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
    // let tty_reader = io::BufReader::new(tty_in);

    let std_lines = std_reader.lines()
        .map(|res| {
            let line = res.expect("Error reading from STDIN");

            Packet::Inbound(line)
        });
        // .chain(stream::once(Packet::Done));

    let tty_chars = inbound(tty_in);

    // This select works in a round robin fashion
    let mut all = futures::stream::select(tty_chars, std_lines);

    while let Some(packet) = all.next().await {
        match packet {
            Packet::Key(Key::Char('\n')) |
            Packet::Key(Key::Alt('\u{0}')) |
            Packet::Key(Key::Esc) => {
                wire.send(Packet::Done).await?;
                break
            },
            _ => wire.send(packet).await?,
        }
    }

    drop(wire);
    drop(all);

    debug!("[input_loop] end");

    Ok(())
}

async fn broker_loop(mut packets: Receiver<Packet>) -> Result<()> {
    debug!("[broker_loop] start");

    // Get all outputs
    // NOTE: If we want to move the output to another task
    //   the State needs to implement Copy and that might be too much
    //   for this scenario (or not)
    let mut tty_out = get_tty().await?;

    // BEGIN: RAW MODE
    let fd = tty_out.as_raw_fd();
    let tty = Termios::from_fd(fd)?;
    let mut raw_tty = tty;
    raw_tty.c_lflag &= !(termios::ICANON | termios::ECHO);
    termios::tcsetattr(fd, termios::TCSANOW, &raw_tty)?;
    // END:   RAW MODE

    let mut state = State::new();

    while let Some(packet) = packets.next().await {
        match packet {
            Packet::Inbound(s) => {
                state.add_string(s);
            },
            Packet::Key(k) => {
                match k {
                    Key::Char(ch) => {
                        state.add_char(ch);
                        debug!("[broker_loop] start fuzzy search");
                        state.search();
                        debug!("[broker_loop] end fuzzy search");
                    },
                    _ => (),
                };
            },
            // Packet::Ignore => (),
            Packet::Done => break,
        };

        let l = format!("query: {:?}\nmatches: {:?}\n", state.query, state.matches);
        tty_out.write_all(l.as_bytes()).await?;
        tty_out.flush().await?;
    };

    debug!("[broker_loop] end");

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
