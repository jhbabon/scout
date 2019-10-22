use log::debug;
use async_std::io;
use async_std::prelude::*;
use futures::channel::mpsc::Receiver;
use crate::result::Result;
use crate::events::Event;
use crate::state::State;
// FIXME: rename these
use crate::output::{Renderer, Layout};

pub async fn task<W>(outbound: W, mut wire: Receiver<Event>) -> Result<Option<String>>
where
    W: io::Write + Send + Unpin + 'static,
{
    debug!("[task] start");

    let mut selection = None;
    let mut should_render: bool;

    let mut state = State::new();
    let mut renderer = Renderer::new(outbound);
    renderer.setup().await?;

    let mut layout = Layout::new();
    layout.update(&state)?;
    renderer.render(&layout).await?;

    while let Some(event) = wire.next().await {
        should_render = false;

        match event {
            Event::Query(query) => {
                state.update_query_string(query);
                should_render = true;
            },
            Event::State(st) => {
                state.matches = st.matches;
                should_render = true;
            },
            Event::Up => {
                state.select_up();
                should_render = true;
            },
            Event::Down => {
                state.select_down();
                should_render = true;
            },
            // NOTE: We don't need to break the loop since
            // the engine and input will drop the sender
            // and the loop will stop
            Event::Done => {
                selection = state.selection();
            },
            _ => (),
        };

        if should_render {
            layout.update(&state)?;
            renderer.render(&layout).await?;
        }
    };

    renderer.teardown().await?;

    drop(wire);

    debug!("[task] end");

    Ok(selection)
}
