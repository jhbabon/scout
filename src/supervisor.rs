use async_std::io;
use async_std::task;
use async_std::future::join;
use futures::channel::mpsc::{self,Sender,Receiver};
use crate::common::{Result, Text};
use crate::config::Config;
use crate::events::Event;
use crate::pipe;
use crate::input;
use crate::engine;
use crate::conveyor;

const CHANNEL_SIZE: usize = 1024;

//*********************************************************************
// Four main tasks:
//
// * pipe: Gets the strings for the search pool
// * input: User input
// * conveyor: How to print the conveyor
// * engine: Search engine
//*********************************************************************
pub async fn run<R,I,W>(config: Config, pipein: R, inbound: I, outbound: W) -> Result<Option<Text>>
where
    R: io::Read + Send + Unpin + 'static,
    I: io::Read + Send + Unpin + 'static,
    W: io::Write + Send + Unpin + 'static,
{
    // wires
    let (pipe_sender, pipe_recv) = wires();
    let (input_sender, input_recv) = wires();
    let (conveyor_sender, conveyor_recv) = wires();

    let pipe_task = task::spawn(pipe::task(config.clone(), pipein, pipe_sender));
    let input_task = task::spawn(input::task(config.clone(), inbound, input_sender, conveyor_sender.clone()));
    let engine_task = task::spawn(engine::task(config.clone(), pipe_recv, input_recv, conveyor_sender));
    let conveyor_task = task::spawn(conveyor::task(config.clone(), outbound, conveyor_recv));

    let (p_res, in_res, en_res, con_res) = join!(
        pipe_task,
        input_task,
        engine_task,
        conveyor_task,
    ).await;

    let _ = p_res?;
    let _ = in_res?;
    let _ = en_res?;

    con_res
}

fn wires() -> (Sender<Event>, Receiver<Event>) {
    mpsc::channel::<Event>(CHANNEL_SIZE)
}
