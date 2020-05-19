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
//! +--------------+                                   +--------+
//! | person_input +---------------------------------->+ screen |
//! +------+-------+                                   +---+----+
//!        |                                               ^
//!        +----------------------+                        |
//!                               |                        |
//!                               v                        |
//! +------------+           +----+-----+                  |
//! | data_input +---------->+  engine  +------------------+
//! +------------+           +----------+
//! ```
//!
//! The person's input is delivered to the engine and the screen at the same time
//! to make the screen as responsive as possible.

use crate::common::{Result, Text};
use crate::config::Config;
use crate::data_input;
use crate::engine;
use crate::events::Event;
use crate::person_input;
use crate::screen;
use async_std::io;
use async_std::sync::{self, Receiver, Sender};
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

    let data_task = task::spawn(data_input::task(stdin, input_sender.clone()));
    let person_task = task::spawn(person_input::task(
        config.clone(),
        inbox,
        input_sender,
        output_sender.clone(),
    ));
    let engine_task = task::spawn(engine::task(input_recv, output_sender));
    let screen_task = task::spawn(screen::task(config, outbox, output_recv));

    let selection = screen_task.await;

    // Stop all remaining tasks
    drop(data_task);
    drop(person_task);
    drop(engine_task);

    selection
}

fn channel() -> (Sender<Event>, Receiver<Event>) {
    sync::channel::<Event>(CHANNEL_SIZE)
}
