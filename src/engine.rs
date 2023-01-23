//! Search engine: Where the fuzzy matching magic happens
//!
//! This task will collect all the input from STDIN and search over them on new queries.
//! Once a search is done all the results will be sent to the screen.

use crate::common::{Result, Text, TextBuilder};
use crate::config::Config;
use crate::events::Event;
use crate::fuzzy;
use async_std::channel::{Receiver, Sender};
use async_std::prelude::*;
use std::collections::VecDeque;

const BUFFER_LIMIT: usize = 5000;

/// Run the search engine task
pub async fn task(
    config: Config,
    mut input_recv: Receiver<Event>,
    output_sender: Sender<Event>,
) -> Result<()> {
    log::trace!("starting search engine");

    let pool_size = config.advanced.pool_size();
    let mut pool: VecDeque<Text> = VecDeque::new();
    let mut count = 0;
    let mut query = String::from("");

    while let Some(event) = input_recv.next().await {
        match event {
            Event::NewLine(s) => {
                log::trace!("line: {:?}", s);

                // Push the new line into the main pool
                pool.push_back(TextBuilder::build(&s));
                count += 1;

                // The pool might be full (too many lines in memory)
                // so we drop the first line
                if pool.len() > pool_size {
                    log::trace!("pool limit ({:?}) exceeded, dropping first line", pool_size);
                    let _f = pool.pop_front();
                }

                // We've got enough lines to refresh the search and send it
                // to the screen
                if count > BUFFER_LIMIT {
                    count = 0;
                    let matches = fuzzy::search(&query, &pool);
                    output_sender
                        .send(Event::Flush((matches, pool.len())))
                        .await?;
                }
            }
            Event::EOF => {
                log::trace!("all input data done");
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
