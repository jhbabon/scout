//! Read lines from STDIN and signal when the STDIN has been consumed

use crate::broadcast::{Broadcaster, Task};
use crate::common::Result;
use crate::events::Event;
use async_std::io;
use async_std::prelude::*;
use async_std::stream;

/// Run the data input task
pub async fn task<R>(stdin: R, sender: Broadcaster) -> Result<()>
where
    R: io::Read + Unpin + Send + 'static,
{
    log::trace!("starting to read input data");

    let reader = io::BufReader::new(stdin);
    let mut stream = reader
        .lines()
        .map(|res| res.expect("Error reading from STDIN"))
        .filter(|line| !line.is_empty())
        .map(Event::NewLine)
        .chain(stream::once(Event::EOF));

    while let Some(event) = stream.next().await {
        sender.send_to(event, Task::Engine).await?;
    }

    log::trace!("input data done");

    Ok(())
}
