//! Print the program's interface and keep track of the selection state
//!
//! The selection state is done here because this task receives both the person's interactions and
//! the results from the search engine. That is, it "knows" what the person sees and why they are
//! moving, typing, etc.
//!
//! When the program finishes this is the task that will return the final person's selection.

use crate::common::{Result, Text};
use crate::config::Config;
use crate::events::Event;
use crate::state::State;
use crate::ui::Canvas;
use async_std::io;
use async_std::prelude::*;
use async_std::sync::Receiver;
use log;
use std::time::Instant;

/// Run the screen's task
pub async fn task<W>(config: Config, outbound: W, mut recv: Receiver<Event>) -> Result<Option<Text>>
where
    W: io::Write + Send + Unpin + 'static,
{
    log::trace!("starting screen");

    let mut last_timestamp = Instant::now();
    let mut render: bool;
    let mut selection = None;

    let mut state = State::new();
    let mut canvas = Canvas::new(&config, outbound).await?;

    canvas.render(&state).await?;

    while let Some(event) = recv.next().await {
        render = false;

        match event {
            Event::Search(prompt) => {
                log::trace!("printing prompt: {:?}", prompt);

                last_timestamp = prompt.timestamp();
                state.set_search(prompt);
                render = true;
            }

            Event::Flush((matches, len)) => {
                log::trace!("flushing matches");

                // Flush happens when the pool size
                // changes or the pool is complete
                state.set_matches((matches, len));
                render = true;
            }

            Event::SearchDone((matches, len, timestamp)) => {
                // Only if the search timestamp is
                // the same as the last query timestamp
                // we will update the state. This way
                // we will drop any intermediate search
                // and reduce the number of renders
                if timestamp >= last_timestamp {
                    log::trace!("printing new search results");

                    state.set_matches((matches, len));
                    render = true;
                }
            }

            Event::Up => {
                log::trace!("moving selection up");

                state.select_up();
                render = true;
            }
            Event::Down => {
                log::trace!("moving selection down");

                state.select_down();
                render = true;
            }

            Event::Done => {
                selection = state.selection();
                break;
            }
            Event::Exit => break,

            _ => (),
        };

        if render {
            canvas.render(&state).await?;
        }
    }

    log::trace!("screen done");

    Ok(selection)
}
