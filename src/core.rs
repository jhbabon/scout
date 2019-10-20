use log::debug;
use async_std::prelude::*;
use futures::channel;
use crate::result::Result;
use crate::ptty::get_ptty;
use crate::events::Event;
// use crate::fuzzy::Candidate;
use crate::state::State;
use crate::output::{Renderer, Layout};

type Receiver<T> = channel::mpsc::Receiver<T>;

pub async fn task(mut events: Receiver<Event>) -> Result<Option<String>> {
    debug!("[task] start");

    let mut selection: Option<String> = None;
    let mut state = State::new();

    let ptty_out = get_ptty().await?;
    let mut renderer = Renderer::new(ptty_out);
    renderer.setup().await?;

    let mut layout = Layout::new();
    layout.update(&state)?;
    renderer.render(&layout).await?;

    while let Some(event) = events.next().await {
        match event {
            Event::Packet(s) => {
                state.add_candidate(s);
            },
            Event::EOF => {
                state.candidates_done();
            },
            Event::Clear => {
                state.clear_query();
            },
            Event::Backspace => {
                state.del_input();
            },
            Event::Input(ch) => {
                state.add_input(ch);
            },
            Event::Up => {
                state.select_up();
            },
            Event::Down => {
                state.select_down();
            },
            Event::Done => {
                selection = state.selection();

                break
            },
            Event::Exit => {
                break
            },
            _ => (),
        };

        // TODO: do it only some times
        state.search();

        layout.update(&state)?;
        renderer.render(&layout).await?;
    };

    renderer.teardown().await?;

    debug!("[task] end");

    Ok(selection)
}
