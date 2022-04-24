// FIXME: Update docs with new task
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

use crate::broadcast::{self, Task};
use crate::common::{Result, Text};
use crate::config::Config;
use crate::data_input;
use crate::engine;
use crate::person_input;
use crate::screen;
use crate::surroundings;
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
    // broadcast channel
    let (sender, receiver) = broadcast::broadband(
        CHANNEL_SIZE,
        &[Task::Screen, Task::Engine, Task::Surroundings],
    );

    let screen_task = task::spawn(screen::task(
        config.clone(),
        outbox,
        sender.on(&[Task::Surroundings])?,
        receiver.on(Task::Screen)?,
    ));
    let engine_task = task::spawn(engine::task(
        sender.on(&[Task::Screen, Task::Surroundings])?,
        receiver.on(Task::Engine)?,
    ));
    let surroundings_task = task::spawn(surroundings::task(
        sender.on(&[Task::Screen])?,
        receiver.on(Task::Surroundings)?,
    ));
    let data_task = task::spawn(data_input::task(stdin, sender.on(&[Task::Engine])?));
    let person_task = task::spawn(person_input::task(config, inbox, sender));

    let selection = screen_task.await;

    // Stop all remaining tasks
    drop(data_task);
    drop(person_task);
    drop(surroundings_task);
    drop(engine_task);

    selection
}
