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

use crate::broadcast::{Broadcaster, Task};
use crate::common::{Prompt, Result};
use crate::config::Config;
use crate::events::Event;
use async_std::io;
use async_std::prelude::*;
use termion::event::Key;
use termion::input::TermRead;

/// Run the person's input task
pub async fn task<R>(config: Config, mut input: R, sender: Broadcaster) -> Result<()>
where
    R: io::Read + Unpin + Send + 'static,
{
    log::trace!("starting to read person's input");

    let mut buffer;
    let mut query_updated: bool;
    let mut prompt: Prompt;

    if let Some(q) = &config.initial_query {
        prompt = q.into();

        sender
            .send_many(Event::Search(prompt.clone()), &[Task::Screen, Task::Engine])
            .await?;
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
                    sender.send_to(Event::Up, Task::Screen).await?;
                }
                Key::Ctrl('n') | Key::Down => {
                    sender.send_to(Event::Down, Task::Screen).await?;
                }

                Key::Esc | Key::Alt('\u{0}') => {
                    sender.send_all(Event::Exit).await?;

                    break 'event;
                }
                Key::Char('\n') => {
                    sender.send_all(Event::Done).await?;

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
                    prompt.add(ch);
                    query_updated = true;
                }

                Key::Left => {
                    prompt.left();
                    sender
                        .send_to(Event::Search(prompt.clone()), Task::Screen)
                        .await?;
                }
                Key::Right => {
                    prompt.right();
                    sender
                        .send_to(Event::Search(prompt.clone()), Task::Screen)
                        .await?;
                }
                Key::Ctrl('a') => {
                    prompt.cursor_at_start();
                    sender
                        .send_to(Event::Search(prompt.clone()), Task::Screen)
                        .await?;
                }
                Key::Ctrl('e') => {
                    prompt.cursor_at_end();
                    sender
                        .send_to(Event::Search(prompt.clone()), Task::Screen)
                        .await?;
                }

                _ => (),
            }
        }

        if query_updated {
            sender
                .send_many(Event::Search(prompt.clone()), &[Task::Screen, Task::Engine])
                .await?;
        }
    }

    log::trace!("person's input done");

    Ok(())
}

fn keys(buffer: &mut [u8], num: usize) -> Vec<Key> {
    let tmp: Vec<u8> = buffer.iter().take(num).copied().collect();

    tmp.keys()
        .filter(|k| k.is_ok())
        .map(|k| k.unwrap())
        .collect()
}
