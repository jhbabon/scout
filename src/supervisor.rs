//! Setup and run the all the tasks that compose the program
//!
//! The program runs over four main tasks
//!
//! * `data_input::task`: Handles input lines from `STDIN`
//! * `person_input::task`: Handles the person's interactions with the program
//! * `engine::task`: The search engine, it performs the actual fuzzy search
//! * `screen::task`: How to print the program's interface
//!
//! All tasks are futures that communicate between them sending events through channels
//! as you can see in the following diagram:
//!
//! ```text
//! +--------------+                    +--------+
//! | person_input +---------+--------->+ screen |
//! +------+-------+         ^          +--------+
//!        |                 |
//!        v             +---+----+
//!        +------------>+ engine |
//!        ^             +--------+
//!        |
//! +------+-----+
//! | data_input |
//! +------------+
//! ```
//!
//! The input from the person using the program is delivered to both the engine and the screen in
//! two different channels. There are some interactions (like moving through the list) that are
//! only relevant to the screen. Others, like new queries, are relevant for both. Using these two
//! channels also makes the screen more responsive to interactions since it doesn't have to wait
//! for the engine to finish searching in order to update the prompt, for example.

use crate::common::{Result, Text};
use crate::config::Config;
use crate::data_input;
use crate::engine;
use crate::events::Event;
use crate::person_input;
use crate::screen;
use async_std::channel::{self, Receiver, Sender};
use async_std::io;
use async_std::task;

const CHANNEL_SIZE: usize = 1024;

/// Run the program's tasks.
pub async fn run<R, I, W>(config: Config, stdin: R, inbox: I, outbox: W) -> Result<Option<Text>>
where
    R: io::Read + Send + Unpin + 'static,
    I: io::Read + Send + Unpin + 'static,
    W: io::Write + Send + Unpin + 'static,
{
    // channels
    let (input_sender, input_recv) = channel();
    let (output_sender, output_recv) = channel();

    let screen_task = task::spawn(screen::task(config.clone(), outbox, output_recv));
    let person_task = task::spawn(person_input::task(
        config.clone(),
        inbox,
        input_sender.clone(),
        output_sender.clone(),
    ));
    let engine_task = task::spawn(engine::task(config, input_recv, output_sender));
    let data_task = task::spawn(data_input::task(stdin, input_sender));

    let selection = screen_task.await;

    // Stop all remaining tasks
    drop(data_task);
    drop(person_task);
    drop(engine_task);

    selection
}

fn channel() -> (Sender<Event>, Receiver<Event>) {
    channel::bounded::<Event>(CHANNEL_SIZE)
}
