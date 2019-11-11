use crate::common::{Result, Text};
use crate::config::Config;
use crate::events::Event;
use crate::state::State;
use crate::ui::Layout;
use async_std::io;
use async_std::prelude::*;
use async_std::sync::Receiver;
use log::debug;
use std::time::Instant;

pub async fn task<W>(
    config: Config,
    outbound: W,
    mut conveyor_recv: Receiver<Event>,
) -> Result<Option<Text>>
where
    W: io::Write + Send + Unpin + 'static,
{
    debug!("[task] start");

    let mut last_timestamp = Instant::now();
    let mut render: bool;
    let mut selection = None;

    let mut state = State::new();
    let mut layout = Layout::new(&config, outbound).await?;

    layout.render(&state).await?;

    while let Some(event) = conveyor_recv.next().await {
        debug!("Got event {:?}", event);

        render = false;

        match event {
            Event::Query((query, timestamp)) => {
                last_timestamp = timestamp;
                state.set_query(query);
                render = true;
            }

            Event::FlushSearch((matches, len)) => {
                // Flush happens when the pool size
                // changes or the pool is complete
                state.set_matches((matches, len));
                render = true;
            }
            Event::Search((matches, len, timestamp)) => {
                // Only if the search timestamp is
                // the same as the last query timestamp
                // we will update the state. This way
                // we will drop any intermediate search
                // and reduce the number of renders
                if timestamp >= last_timestamp {
                    state.set_matches((matches, len));
                    render = true;
                }
            }

            Event::Up => {
                state.select_up();
                render = true;
            }
            Event::Down => {
                state.select_down();
                render = true;
            }

            // NOTE: We don't need to break the loop since
            // the engine and input will drop the sender
            // and the loop will stop
            Event::Done => {
                selection = state.selection();
            }
            _ => (),
        };

        if render {
            layout.render(&state).await?;
        }
    }

    drop(conveyor_recv);

    debug!("[task] end");

    Ok(selection)
}
