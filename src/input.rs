use log::debug;
use async_std::prelude::*;
use async_std::io;
use futures::SinkExt;
use futures::channel::mpsc::Sender;
use termion::input::TermRead;
use termion::event::Key;
use crate::config::Config;
use crate::common::Result;
use crate::events::Event;

pub async fn task<R>(config: Config, mut inbound: R, mut engine_wire: Sender<Event>, mut screen_wire: Sender<Event>) -> Result<()>
where
    R: io::Read + Unpin + Send + 'static
{
    debug!("[task] start");

    let mut buffer;
    let mut query: Vec<char> = vec![];
    let mut query_updated: bool;

    if let Some(q) = config.initial_query {
        let q = q.to_string();
        screen_wire.send(Event::Query(q.clone())).await?;
        engine_wire.send(Event::Query(q)).await?;
    }

    'event: loop {
        debug!("[task|event loop] iteration");

        buffer = vec![0; 10];
        query_updated = false;

        let fut = inbound.read(&mut buffer);
        let num = fut.await?;
        let keys = keys(&mut buffer, num);
        let mut keys = keys.iter();

        while let Some(key) = keys.next() {
            match key {
                Key::Ctrl('p') | Key::Up => {
                    screen_wire.send(Event::Up).await?;
                },
                Key::Ctrl('n') | Key::Down => {
                    screen_wire.send(Event::Down).await?;
                },

                Key::Esc | Key::Alt('\u{0}') => {
                    engine_wire.send(Event::Exit).await?;
                    screen_wire.send(Event::Exit).await?;

                    break 'event;
                },
                Key::Char('\n') => {
                    screen_wire.send(Event::Done).await?;
                    engine_wire.send(Event::Done).await?;

                    break 'event;
                },

                Key::Ctrl('u') => {
                    query.clear();
                    query_updated = true;
                },
                Key::Backspace => {
                    let _p = query.pop();
                    query_updated = true;
                },
                Key::Char(ch) => {
                    query.push(ch.clone());
                    query_updated = true;
                },

                _ => (),
            }
        }

        if query_updated {
            let q: String = query.iter().collect();

            debug!("[task|event loop] sending query {:?}", q);

            screen_wire.send(Event::Query(q.clone())).await?;
            engine_wire.send(Event::Query(q)).await?;
        }
    }

    drop(engine_wire);
    drop(screen_wire);

    debug!("[task] end");

    Ok(())
}

fn keys(buffer: &mut Vec<u8>, num: usize) -> Vec<Key> {
    let tmp: Vec<u8> = buffer
        .iter()
        .take(num)
        .cloned()
        .collect();

    tmp.keys()
        .filter(|k| k.is_ok())
        .map(|k| k.unwrap())
        .collect()
}
