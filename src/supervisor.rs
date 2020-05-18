use crate::common::{Result, Text};
use crate::config::Config;
use crate::screen;
use crate::engine;
use crate::events::Event;
use crate::person_input;
use crate::data_input;
use async_std::io;
use async_std::sync::{channel, Receiver, Sender};
use async_std::task;

const CHANNEL_SIZE: usize = 1024;

/// Setup and run the four main tasks that compose the program
pub async fn run<R, I, W>(
    config: Config,
    pipein: R,
    inbound: I,
    outbound: W,
) -> Result<Option<Text>>
where
    R: io::Read + Send + Unpin + 'static,
    I: io::Read + Send + Unpin + 'static,
    W: io::Write + Send + Unpin + 'static,
{
    // wires
    let (input_sender, input_recv) = wire();
    let (output_sender, output_recv) = wire();

    let data_task = task::spawn(data_input::task(pipein, input_sender.clone()));
    let person_task = task::spawn(person_input::task(config.clone(), inbound, input_sender, output_sender.clone()));
    let engine_task = task::spawn(engine::task(input_recv, output_sender));
    let screen_task = task::spawn(screen::task(config, outbound, output_recv));

    let selection = screen_task.await;

    drop(data_task);
    drop(person_task);
    drop(engine_task);

    selection
}

fn wire() -> (Sender<Event>, Receiver<Event>) {
    channel::<Event>(CHANNEL_SIZE)
}
