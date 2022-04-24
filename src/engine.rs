//! Search engine: Where the fuzzy matching magic happens
//!
//! This task will collect all the input from STDIN and search over them on new queries.
//! Once a search is done all the results will be sent to the screen.

use crate::broadcast::{Broadcaster, Task};
use crate::common::{Pool, PoolBuilder, Result, Text, TextBuilder};
use crate::events::Event;
use crate::fuzzy;
use async_std::channel::Receiver;
use async_std::prelude::*;

const BUFFER_LIMIT: usize = 5000;
const POOL_LIMIT: usize = 50000;

/// Run the search engine task
pub async fn task(sender: Broadcaster, mut receiver: Receiver<Event>) -> Result<()> {
    log::trace!("starting search engine");

    let pool: Pool<Text> = PoolBuilder::with_capacity(POOL_LIMIT);
    let mut overflow: Vec<Text> = Vec::with_capacity(BUFFER_LIMIT);
    let mut pool_count = 0;
    let mut buffer_count = 0;
    let mut query = String::from("");

    while let Some(event) = receiver.next().await {
        match event {
            Event::NewLine(s) => {
                log::trace!("line: {:?}", s);

                let text = TextBuilder::build(&s);
                let mut pl = pool.write().await;
                if pool_count < POOL_LIMIT {
                    buffer_count += 1;
                    pool_count += 1;
                    pl.push(text);

                    sender
                        .send_to(Event::Pool(pool.clone()), Task::Surroundings)
                        .await?;
                } else {
                    // If we get more elements than the pool limit we save
                    // them in an overflow vec.
                    overflow.push(text);

                    if overflow.len() == BUFFER_LIMIT {
                        // If the overflow vec is full we can assign
                        // these elements to the main pool
                        pl.drain(0..BUFFER_LIMIT);
                        pl.append(&mut overflow);
                        buffer_count = BUFFER_LIMIT;

                        sender
                            .send_to(Event::Pool(pool.clone()), Task::Surroundings)
                            .await?;
                    }
                }

                // We've got enough lines to refresh the search and send it
                // to the screen
                if buffer_count >= BUFFER_LIMIT {
                    buffer_count = 0;
                    let matches = fuzzy::search(&query, &pl);
                    sender
                        .send_to(Event::Flush((matches, pl.len())), Task::Screen)
                        .await?;
                }
            }
            Event::EOF => {
                log::trace!("all input data done");
                if !overflow.is_empty() {
                    let mut pl = pool.write().await;
                    pl.drain(0..overflow.len());
                    pl.append(&mut overflow);
                }

                let pl = pool.read().await;
                let matches = fuzzy::search(&query, &pl);
                sender
                    .send_to(Event::Flush((matches, pl.len())), Task::Screen)
                    .await?;

                sender
                    .send_to(Event::Pool(pool.clone()), Task::Surroundings)
                    .await?;
            }
            Event::Search(prompt) => {
                query = prompt.as_string();
                log::trace!("performing new search: '{}'", query);

                let pl = pool.read().await;
                let matches = fuzzy::search(&query, &pl);
                let results = Event::SearchDone((matches, pl.len(), prompt.timestamp()));

                sender.send_to(results, Task::Screen).await?;
            }
            Event::Done | Event::Exit => break,
            _ => (),
        };
    }

    log::trace!("search engine done");

    Ok(())
}
