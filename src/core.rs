use log::debug;
use async_std::prelude::*;
use futures::channel;
use crate::result::Result;
use crate::ptty::get_ptty;
use crate::events::Event;
// use crate::fuzzy::Candidate;
use crate::state::State;
use crate::output::Output;

type Receiver<T> = channel::mpsc::UnboundedReceiver<T>;

pub async fn task(mut events: Receiver<Event>) -> Result<Option<String>> {
    debug!("[task] start");

    // Get all outputs
    // NOTE: If we want to move the output to another task
    //   the State needs to implement Copy and that might be too much
    //   for this scenario (or not)
    let ptty_out = get_ptty().await?;
    let mut output = Output::new(ptty_out);
    output.setup().await?;

    let mut exit_event: Event = Event::Ignore;
    let mut state = State::new();

    while let Some(event) = events.next().await {
        match event {
            Event::Packet(s) => {
                state.add_string(s);
            },
            Event::Input(ch) => {
                state.add_char(ch);
                debug!("[task] start fuzzy search");
                state.search();
                debug!("[task] end fuzzy search");
            },
            Event::Done | Event::Exit => {
                exit_event = event;
                break
            },
            _ => (),
        };

        let l = format!("query: {:?}\nmatches: {:?}\n", state.query, state.matches);
        output.render(l).await?;
    };

    output.teardown().await?;

    debug!("[task] end");

    match exit_event {
        Event::Done => {
            match state.matches.first() {
                Some(candidate) => Ok(Some(candidate.string.clone())),
                None => Ok(None)
            }
        },
        _ => Ok(None),
    }
}
