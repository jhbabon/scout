//! Search engine: Where the fuzzy matching magic happens
//!
//! This task will collect all the input from STDIN and search over them on new queries.
//! Once a search is done all the results will be sent to the screen.

use crate::common::{Result, Text, TextBuilder};
use crate::events::Event;
use crate::fuzzy;
use async_std::channel::{Receiver, Sender};
use async_std::prelude::*;

const BUFFER_LIMIT: usize = 5000;
const POOL_LIMIT: usize = 50000;

/// Run the search engine task
pub async fn task(mut input_recv: Receiver<Event>, output_sender: Sender<Event>) -> Result<()> {
    log::trace!("starting search engine");

    let mut pool: Vec<Text> = Vec::with_capacity(POOL_LIMIT);
    let mut overflow: Vec<Text> = Vec::with_capacity(BUFFER_LIMIT);
    let mut pool_count = 0;
    let mut buffer_count = 0;
    let mut query = String::from("");

    while let Some(event) = input_recv.next().await {
        match event {
            Event::NewLine(s) => {
                log::trace!("line: {:?}", s);

                let text = TextBuilder::build(&s);
                if pool_count < POOL_LIMIT {
                    buffer_count += 1;
                    pool_count += 1;
                    pool.push(text);
                } else {
                    // If we get more elements than the pool limit we save
                    // them in an overflow vec.
                    overflow.push(text);

                    if overflow.len() == BUFFER_LIMIT {
                        // If the overflow vec is full, we can assign
                        // these elements to the main pool
                        pool.drain(0..BUFFER_LIMIT);
                        pool.append(&mut overflow);
                        buffer_count = BUFFER_LIMIT;
                    }
                }

                // We've got enough lines to refresh the search and send it
                // to the screen
                if buffer_count >= BUFFER_LIMIT {
                    buffer_count = 0;
                    let matches = fuzzy::search(&query, &pool);
                    output_sender
                        .send(Event::Flush((matches, pool.len())))
                        .await?;
                }
            }
            Event::EOF => {
                log::trace!("all input data done");
                if !overflow.is_empty() {
                    pool.drain(0..overflow.len());
                    pool.append(&mut overflow);
                }
                let matches = fuzzy::search(&query, &pool);
                output_sender
                    .send(Event::Flush((matches, pool.len())))
                    .await?;
            }
            Event::Search(prompt) => {
                query = prompt.as_string();
                log::trace!("performing new search: '{}'", query);

                let matches = fuzzy::search(&query, &pool);
                let results = Event::SearchDone((matches, pool.len(), prompt.timestamp()));

                output_sender.send(results).await?;
            }
            Event::Done | Event::Exit => break,
            _ => (),
        };
    }

    log::trace!("search engine done");

    Ok(())
}
