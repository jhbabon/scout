use crate::common::Result;
use crate::events::Event;
use async_std::io;
use async_std::prelude::*;
use async_std::stream;
use async_std::sync::Sender;
use log;

pub async fn task<R>(pipe: R, sender: Sender<Event>) -> Result<()>
where
    R: io::Read + Unpin + Send + 'static,
{
    log::trace!("starting to read input data");

    let reader = io::BufReader::new(pipe);
    let mut stream = reader
        .lines()
        .map(|res| res.expect("Error reading from PIPE"))
        .filter(|line| !line.is_empty())
        .map(|line| Event::NewLine(line))
        .chain(stream::once(Event::EOF));

    while let Some(event) = stream.next().await {
        sender.send(event).await;
    }

    log::trace!("input data done");

    Ok(())
}
