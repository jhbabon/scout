use crate::common::{Result, Text};
use crate::config::Config;
use crate::conveyor;
use crate::engine;
use crate::events::Event;
use crate::input;
use crate::pipe;
use async_std::io;
use async_std::sync::{channel, Receiver, Sender};
use async_std::task;

const CHANNEL_SIZE: usize = 1024;

//*********************************************************************
// Four main tasks:
//
// * pipe: Gets the strings for the search pool
// * input: User input
// * conveyor: How to print the screen
// * engine: Search engine
//*********************************************************************
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
    let (input_sender, input_recv) = wires();
    let (output_sender, output_recv) = wires();

    let pipe_task = task::spawn(pipe::task(config.clone(), pipein, input_sender.clone()));
    let input_task = task::spawn(input::task(config.clone(), inbound, input_sender));
    let engine_task = task::spawn(engine::task(config.clone(), input_recv, output_sender));
    let conveyor_task = task::spawn(conveyor::task(config, outbound, output_recv));

    let result = conveyor_task.await;

    // TODO: Review drop usage, I don't think I need it so much
    drop(pipe_task);
    drop(input_task);
    drop(engine_task);

    result
}

fn wires() -> (Sender<Event>, Receiver<Event>) {
    channel::<Event>(CHANNEL_SIZE)
}
