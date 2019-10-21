use log::debug;
use async_std::prelude::*;
use async_std::io;
use async_std::stream;
use futures::{channel, SinkExt};
use crate::result::Result;
use crate::events::Event;

type Sender<T> = channel::mpsc::Sender<T>;

pub async fn task(mut wire: Sender<Event>) -> Result<()> {
    debug!("[task] start");

    let stdin = io::stdin();

    let std_reader = io::BufReader::new(stdin);
    let mut std_stream = std_reader.lines()
        .map(|res| {
            let line = res.expect("Error reading from STDIN");

            Event::Packet(line)
        })
        .chain(stream::once(Event::EOF));

    while let Some(event) = std_stream.next().await {
        if let Err(_) = wire.send(event).await {
            break;
        }
    }

    drop(wire);

    debug!("[task] end");

    Ok(())
}
