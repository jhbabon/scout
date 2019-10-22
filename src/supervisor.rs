use async_std::io;
use async_std::task;
use async_std::future::join;
use futures::channel::mpsc::{self,Sender,Receiver};
use crate::result::Result;
use crate::events::Event;
use crate::pipe;
use crate::input;
use crate::engine;
use crate::screen;

const CHANNEL_SIZE: usize = 1024;

//*********************************************************************
// Four main tasks:
//
// * pipe: Gets the strings for the search pool
// * input: User input
// * screen: How to print the screen
// * engine: Search engine
//*********************************************************************
pub async fn run<R,I,W>(pipein: R, inbound: I, outbound: W) -> Result<Option<String>>
where
    R: io::Read + Send + Unpin + 'static,
    I: io::Read + Send + Unpin + 'static,
    W: io::Write + Send + Unpin + 'static,
{
    // wires
    let (pipe_sender, pipe_recv) = wires();
    let (input_sender, input_recv) = wires();
    let (screen_sender, screen_recv) = wires();

    let pipe_task = task::spawn(pipe::task(pipein, pipe_sender));
    let input_task = task::spawn(input::task(inbound, input_sender, screen_sender.clone()));
    let engine_task = task::spawn(engine::task(pipe_recv, input_recv, screen_sender));
    let screen_task = task::spawn(screen::task(outbound, screen_recv));

    let (p_res, in_res, en_res, sc_res) = join!(
        pipe_task,
        input_task,
        engine_task,
        screen_task,
    ).await;

    let _ = p_res?;
    let _ = in_res?;
    let _ = en_res?;

    sc_res
}

fn wires() -> (Sender<Event>, Receiver<Event>) {
    mpsc::channel::<Event>(CHANNEL_SIZE)
}
