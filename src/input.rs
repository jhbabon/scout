use crate::common::{Result, SearchBox};
use crate::config::Config;
use crate::events::Event;
use async_std::io;
use async_std::prelude::*;
use async_std::sync::Sender;
use log::debug;
use termion::event::Key;
use termion::input::TermRead;

pub async fn task<R>(
    config: Config,
    mut inbound: R,
    input_sender: Sender<Event>,
    conveyor_sender: Sender<Event>,
) -> Result<()>
where
    R: io::Read + Unpin + Send + 'static,
{
    debug!("[task] start");

    let mut buffer;
    let mut query_updated: bool;
    let mut search_box = SearchBox::default();

    if let Some(q) = &config.initial_query {
        search_box = q.into();

        conveyor_sender
            .send(Event::Request(search_box.clone()))
            .await;
        input_sender.send(Event::Request(search_box.clone())).await;
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
                    conveyor_sender.send(Event::Up).await;
                }
                Key::Ctrl('n') | Key::Down => {
                    conveyor_sender.send(Event::Down).await;
                }

                Key::Esc | Key::Alt('\u{0}') => {
                    input_sender.send(Event::Exit).await;
                    conveyor_sender.send(Event::Exit).await;

                    break 'event;
                }
                Key::Char('\n') => {
                    input_sender.send(Event::Done).await;
                    conveyor_sender.send(Event::Done).await;

                    break 'event;
                }

                Key::Ctrl('u') => {
                    search_box.clear();
                    query_updated = true;
                }
                Key::Backspace => {
                    search_box.backspace();
                    query_updated = true;
                }
                Key::Char(ch) => {
                    search_box.add(ch.clone());
                    query_updated = true;
                }

                Key::Left => {
                    search_box.left();
                    query_updated = true;
                }
                Key::Right => {
                    search_box.right();
                    query_updated = true;
                }
                Key::Ctrl('a') => {
                    search_box.to_start();
                    query_updated = true;
                }
                Key::Ctrl('e') => {
                    search_box.to_end();
                    query_updated = true;
                }

                _ => (),
            }
        }

        if query_updated {
            search_box.refresh();
            conveyor_sender
                .send(Event::Request(search_box.clone()))
                .await;
            input_sender.send(Event::Request(search_box.clone())).await;
        }
    }

    drop(input_sender);
    drop(conveyor_sender);

    debug!("[task] end");

    Ok(())
}

fn keys(buffer: &mut Vec<u8>, num: usize) -> Vec<Key> {
    let tmp: Vec<u8> = buffer.iter().take(num).cloned().collect();

    tmp.keys()
        .filter(|k| k.is_ok())
        .map(|k| k.unwrap())
        .collect()
}
