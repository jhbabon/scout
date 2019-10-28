use log::debug;
use std::time::Instant;
use async_std::prelude::*;
use async_std::io;
use futures::SinkExt;
use futures::channel::mpsc::Sender;
use termion::input::TermRead;
use termion::event::Key;
use crate::config::Config;
use crate::common::Result;
use crate::events::Event;

pub async fn task<R>(config: Config, mut inbound: R, mut engine_wire: Sender<Event>, mut conveyor_wire: Sender<Event>) -> Result<()>
where
    R: io::Read + Unpin + Send + 'static
{
    debug!("[task] start");

    let mut buffer;
    let mut query: Vec<char> = vec![];
    let mut query_updated: bool;

    if let Some(q) = config.initial_query {
        let now = Instant::now();
        query = q.chars().collect();

        conveyor_wire.send(Event::Query((q.clone(), now))).await?;
        engine_wire.send(Event::Query((q, now))).await?;
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
                    conveyor_wire.send(Event::Up).await?;
                },
                Key::Ctrl('n') | Key::Down => {
                    conveyor_wire.send(Event::Down).await?;
                },

                Key::Esc | Key::Alt('\u{0}') => {
                    engine_wire.send(Event::Exit).await?;
                    conveyor_wire.send(Event::Exit).await?;

                    break 'event;
                },
                Key::Char('\n') => {
                    conveyor_wire.send(Event::Done).await?;
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
            let now = Instant::now();
            let q: String = query.iter().collect();

            debug!("[task|event loop] sending query {:?}", q);

            conveyor_wire.send(Event::Query((q.clone(), now))).await?;
            engine_wire.send(Event::Query((q, now))).await?;
        }
    }

    drop(engine_wire);
    drop(conveyor_wire);

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
