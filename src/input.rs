use log::debug;
use std::pin::Pin;
use std::collections::VecDeque;
use async_std::prelude::*;
use async_std::io;
use async_std::task::{Context, Poll};
use futures::SinkExt;
use futures::channel::mpsc::Sender;
use termion::input::TermRead;
use termion::event::Key;
use crate::result::Result;
use crate::events::Event;

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

pub async fn task<R>(inbound: R, mut engine_wire: Sender<Event>, mut screen_wire: Sender<Event>) -> Result<()>
where
    R: io::Read + Unpin + Send + 'static
{
    debug!("[task] start");

    let mut query = vec![];
    let mut stream = Interactions::new(inbound);

    while let Some(event) = stream.next().await {
        match event {
            Event::Exit | Event::Done => {
                engine_wire.send(event.clone()).await?;
                screen_wire.send(event).await?;

                break
            },
            Event::Input(ch) => {
                query.push(ch);
                let q: String = query.iter().collect();

                screen_wire.send(Event::Query(q.clone())).await?;
                engine_wire.send(Event::Query(q)).await?;
            },
            Event::Backspace => {
                let _p = query.pop();
                let q: String = query.iter().collect();

                screen_wire.send(Event::Query(q.clone())).await?;
                engine_wire.send(Event::Query(q)).await?;
            },
            Event::Clear => {
                query.clear();
                let q = "".to_string();

                screen_wire.send(Event::Query(q.clone())).await?;
                engine_wire.send(Event::Query(q)).await?;
            },
            _ => {
                screen_wire.send(event).await?;
            },
        }
    }

    drop(engine_wire);
    drop(screen_wire);

    debug!("[task] end");

    Ok(())
}
