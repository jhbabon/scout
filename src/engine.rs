use log::debug;
use async_std::prelude::*;
use futures::stream::select;
use futures::SinkExt;
use futures::channel::mpsc::{Receiver,Sender};
use crate::result::Result;
use crate::events::Event;
use crate::state::State;

// const BUFFER_LIMIT: usize = 5000;
// const POOL_LIMIT: usize = 100000;

pub async fn task(pipe: Receiver<Event>, input: Receiver<Event>, mut screen: Sender<Event>) -> Result<()> {
    debug!("[task] start");

    // FIXME: Remove state struct
    let mut state = State::new();
    let mut count = 0;

    let mut incoming = select(input, pipe);

    while let Some(event) = incoming.next().await {
        match event {
            Event::Packet(s) => {
                state.add_candidate(s);
                count += 1;

                if count > 5000 {
                    count = 0;
                    state.search();
                    screen.send(Event::State(state.clone())).await?;
                }
            },
            Event::EOF => {
                state.search();
                screen.send(Event::State(state.clone())).await?;
            },
            Event::Query(query) => {
                state.update_query_string(query);
                state.search();
                screen.send(Event::State(state.clone())).await?;
            },
            Event::Done | Event::Exit => {
                break
            },
            _ => (),
        };
    };

    drop(screen);
    drop(incoming);

    debug!("[task] end");

    Ok(())
}
