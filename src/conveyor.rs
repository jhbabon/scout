use log::debug;
use async_std::io;
use async_std::prelude::*;
use futures::channel::mpsc::Receiver;
use crate::common::{Result,Text};
use crate::events::Event;

// TODO: Keep using state, but simplify it?
use crate::state::State;
use crate::screen::Screen;

// TODO: Move changes in output::Layout to ui::Layout
use crate::ui::Layout;

pub async fn task<W>(outbound: W, mut wire: Receiver<Event>) -> Result<Option<Text>>
where
    W: io::Write + Send + Unpin + 'static,
{
    debug!("[task] start");

    let mut render: bool;
    let mut screen = Screen::new(outbound).await?;
    let mut selection = None;

    let mut state = State::new();
    let mut layout = Layout::new();
    // TODO: Rendering only parts is too hard, let's rerender everything
    //   as before
    layout.update(&state)?;
    screen.render(&layout).await?;

    while let Some(event) = wire.next().await {
        render = false;
        match event {
            Event::Query(query) => {
                state.update_query(query);
                render = true;
            },
            Event::Matches(matches) => {
                state.update_matches(matches);
                render = true;
            },
            Event::Up => {
                state.select_up();
                render = true;
            },
            Event::Down => {
                state.select_down();
                render = true;
            },

            // NOTE: We don't need to break the loop since
            // the engine and input will drop the sender
            // and the loop will stop
            Event::Done => {
                selection = state.selection();
            },
            _ => (),
        };

        if render {
            layout.update(&state)?;
            screen.render(&layout).await?;
        }
    };

    drop(wire);

    debug!("[task] end");

    Ok(selection)
}
