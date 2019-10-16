use log::debug;

use std::pin::Pin;
use std::collections::VecDeque;

use async_std::prelude::*;
use async_std::io;
use async_std::stream;
use async_std::task::{Context, Poll};
use async_std::os::unix::io::AsRawFd;

use futures::{channel, SinkExt};

use termion::input::TermRead;
use termion::event::Key;

use crate::result::Result;
use crate::tty::{get_tty, into_raw_input};
use crate::events::Event;

type Sender<T> = channel::mpsc::UnboundedSender<T>;

struct Interactions<R> {
    reader: R,
    buffer: VecDeque<Event>,
}

impl<R: io::Read + Unpin> Interactions<R> {
    fn new(r: R) -> Self {
        Self {
            reader: r,
            buffer: VecDeque::new(),
        }
    }
}

impl<R: io::Read + Unpin> Stream for Interactions<R> {
    type Item = Event;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {

        if let Some(event) = self.buffer.pop_front() {
            return Poll::Ready(Some(event));
        }

        let mut buf = vec![0; 4];
        let mut fut = self.reader.read(&mut buf);

        match Pin::new(&mut fut).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(n)) => {
                debug!("[Interactions.poll_next()] bytes read: {:?}", n);

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
                debug!("[Interactions.poll_next()] key: {:?}", key);

                while let Some(k) = keys.next() {
                    match k {
                        Key::Null => (),
                        _ => self.buffer.push_back(Event::from(k)),
                    }
                }
                debug!("[Interactions.poll_next()] extra keys: {:?}", self.buffer);

                Poll::Ready(key.map(|k| Event::from(k)))
            },
            Poll::Ready(Err(_)) => {
                Poll::Ready(None)
            },
        }
    }
}

pub async fn task(mut wire: Sender<Event>) -> Result<()> {
    debug!("[task] start");

    let stdin = io::stdin();
    let tty_in = get_tty().await?;

    into_raw_input(tty_in.as_raw_fd())?;

    let std_reader = io::BufReader::new(stdin);
    let std_stream = std_reader.lines()
        .map(|res| {
            let line = res.expect("Error reading from STDIN");

            Event::Packet(line)
        })
        .chain(stream::once(Event::EOF));

    let tty_stream = interactions(tty_in);

    // This select works in a round robin fashion
    let mut all = futures::stream::select(tty_stream, std_stream);

    while let Some(event) = all.next().await {
        match event {
            Event::Exit | Event::Done => {
                wire.send(event).await?;
                break
            },
            _ => wire.send(event).await?,
        }
    }

    drop(wire);
    drop(all);

    debug!("[task] end");

    Ok(())
}

fn interactions<R: io::Read + Unpin>(reader: R) -> Interactions<R> {
    Interactions::new(reader)
}
