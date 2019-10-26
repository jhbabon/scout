use log::debug;
use async_std::prelude::*;
use async_std::io;
use async_std::stream;
use futures::SinkExt;
use futures::channel::mpsc::Sender;
use crate::common::Result;
use crate::config::Config;
use crate::events::Event;

pub async fn task<R>(_config: Config, pipein: R, mut wire: Sender<Event>) -> Result<()>
where
    R: io::Read + Unpin + Send + 'static
{
    debug!("[task] start");

    let reader = io::BufReader::new(pipein);
    let mut stream = reader.lines()
        .map(|res| {
            let line = res.expect("Error reading from PIPE");

            Event::Packet(line)
        })
        .chain(stream::once(Event::EOF));

    while let Some(event) = stream.next().await {
        if let Err(_) = wire.send(event).await {
            break;
        }
    }

    drop(wire);

    debug!("[task] end");

    Ok(())
}
