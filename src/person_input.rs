//! Read the input from the person using the program
//!
//! The input can be:
//! * New characters for the search query
//! * Control sequences to move around the list
//! * Control sequences to move around the prompt
//! * Signals to cancel and exit
//! * Signals to select a candidate from the list
//!
//! All of these actions are processed and sent as events to
//! the rest of tasks.
//!
//! ### Moving around the list
//!
//! * You can use `Up` and `Down` keys to move through the list of candidates
//! * `<C-p>` does the same the Up key and `<C-n>` as the Down key
//! * `Backspace` will remove the character behind the cursor
//!
//! ### Moving around the prompt
//!
//! * You can use the `Left` and `Right` keys to move the cursor in the prompt
//! * `<C-e>` will go to the end of the prompt and `<C-a>` to the beginning
//! * `<C-u>` clears the current query
//!
//! ### Selecting a candidate and exiting the program
//! * `Enter` will select the current candidate
//! * `Esc` will exit the program without making a selection

use crate::common::{Prompt, Result};
use crate::config::Config;
use crate::events::Event;
use async_std::io;
use async_std::prelude::*;
use async_std::sync::Sender;
use log;
use termion::event::Key;
use termion::input::TermRead;

/// Run the person's input task
pub async fn task<R>(
    config: Config,
    mut input: R,
    engine_sender: Sender<Event>,
    screen_sender: Sender<Event>,
) -> Result<()>
where
    R: io::Read + Unpin + Send + 'static,
{
    log::trace!("starting to read person's input");

    let mut buffer;
    let mut query_updated: bool;
    let mut prompt: Prompt;

    if let Some(q) = &config.initial_query {
        prompt = q.into();

        engine_sender.send(Event::Search(prompt.clone())).await;
        screen_sender.send(Event::Search(prompt.clone())).await;
    } else {
        prompt = Default::default();
    }

    'event: loop {
        buffer = vec![0; 10];
        query_updated = false;

        let num = input.read(&mut buffer).await?;
        let keys = keys(&mut buffer, num);

        for key in keys {
            match key {
                Key::Ctrl('p') | Key::Up => {
                    screen_sender.send(Event::Up).await;
                }
                Key::Ctrl('n') | Key::Down => {
                    screen_sender.send(Event::Down).await;
                }

                Key::Esc | Key::Alt('\u{0}') => {
                    screen_sender.send(Event::Exit).await;
                    engine_sender.send(Event::Exit).await;

                    break 'event;
                }
                Key::Char('\n') => {
                    screen_sender.send(Event::Done).await;
                    engine_sender.send(Event::Done).await;

                    break 'event;
                }

                Key::Ctrl('u') => {
                    prompt.clear();
                    query_updated = true;
                }
                Key::Backspace => {
                    query_updated = prompt.backspace();
                }
                Key::Char(ch) => {
                    prompt.add(ch.clone());
                    query_updated = true;
                }

                Key::Left => {
                    prompt.left();
                    screen_sender.send(Event::Search(prompt.clone())).await;
                }
                Key::Right => {
                    prompt.right();
                    screen_sender.send(Event::Search(prompt.clone())).await;
                }
                Key::Ctrl('a') => {
                    prompt.to_start();
                    screen_sender.send(Event::Search(prompt.clone())).await;
                }
                Key::Ctrl('e') => {
                    prompt.to_end();
                    screen_sender.send(Event::Search(prompt.clone())).await;
                }

                _ => (),
            }
        }

        if query_updated {
            screen_sender.send(Event::Search(prompt.clone())).await;
            engine_sender.send(Event::Search(prompt.clone())).await;
        }
    }

    log::trace!("person's input done");

    Ok(())
}

fn keys(buffer: &mut Vec<u8>, num: usize) -> Vec<Key> {
    let tmp: Vec<u8> = buffer.iter().take(num).map(|i| *i).collect();

    tmp.keys()
        .filter(|k| k.is_ok())
        .map(|k| k.unwrap())
        .collect()
}
