use log::debug;
use async_std::prelude::*;
use async_std::io;
use async_std::stream;
use async_std::sync::Sender;
use crate::common::Result;
use crate::config::Config;
use crate::events::Event;

pub async fn task<R>(_config: Config, pipein: R, pipe_sender: Sender<Event>) -> Result<()>
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
        debug!("Got event {:?}", event);

        match event {
            Event::Packet(_) => pipe_sender.send(event).await,
            Event::EOF => {
                pipe_sender.send(event).await;
                break;
            },
            _ => (),
        }
    }

    drop(pipe_sender);

    debug!("[task] end");

    Ok(())
}
