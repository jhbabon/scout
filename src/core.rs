use log::debug;
use async_std::prelude::*;
use futures::channel;
use crate::result::Result;
use crate::ptty::get_ptty;
use crate::events::Event;
use crate::state::State;
use crate::output::{Renderer, Layout};

type Receiver<T> = channel::mpsc::Receiver<T>;

pub async fn task(interactions: Receiver<Event>, input: Receiver<Event>) -> Result<Option<String>> {
    debug!("[task] start");

    let mut should_render: bool;
    let mut count = 0;

    let mut selection: Option<String> = None;
    let mut state = State::new();

    let ptty_out = get_ptty().await?;
    let mut renderer = Renderer::new(ptty_out);
    renderer.setup().await?;

    let mut layout = Layout::new();
    layout.update(&state)?;
    renderer.render(&layout).await?;

    let mut all = futures::stream::select(interactions, input);

    while let Some(event) = all.next().await {
        should_render = false;

        match event {
            Event::Packet(s) => {
                state.add_candidate(s);
                count += 1;

                if count > 5000 {
                    count = 0;
                    should_render = true;
                    state.search();
                }
            },
            Event::EOF => {
                state.candidates_done();
                should_render = true;
                state.search();
            },
            Event::Clear => {
                state.clear_query();
                should_render = true;
                state.search();
            },
            Event::Backspace => {
                state.del_input();
                should_render = true;
                state.search();
            },
            Event::Input(ch) => {
                state.add_input(ch);
                should_render = true;
                state.search();
            },
            Event::Up => {
                state.select_up();
                should_render = true;
                state.search();
            },
            Event::Down => {
                state.select_down();
                should_render = true;
                state.search();
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

        if should_render {
            layout.update(&state)?;
            renderer.render(&layout).await?;
        }
    };

    renderer.teardown().await?;

    debug!("[task] end");

    Ok(selection)
}
