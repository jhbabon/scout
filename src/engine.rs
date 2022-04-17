//! Search engine: Where the fuzzy matching magic happens
//!
//! This task will collect all the input from STDIN and search over them on new queries.
//! Once a search is done all the results will be sent to the screen.

use crate::common::{Result, Text, TextBuilder};
use crate::events::Event;
use crate::fuzzy;
use async_std::channel::{Receiver, Sender};
use async_std::prelude::*;
use std::collections::VecDeque;

const BUFFER_LIMIT: usize = 5000;
const POOL_LIMIT: usize = 50000;

#[derive(Debug, Default)]
struct SelectionRef {
    index: usize,
    len: usize,
}

impl SelectionRef {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn up(&mut self) {
        if self.index == 0 {
            self.index = self.max();
        } else {
            self.index -= 1;
        }
    }

    pub fn down(&mut self) {
        if self.index == self.max() {
            self.index = 0;
        } else {
            self.index += 1;
        }
    }

    pub fn adjust(&mut self, len: usize) {
        self.len = len;
        if self.index > self.max() {
            self.index = self.max();
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    fn max(&self) -> usize {
        if self.len == 0 {
            0
        } else {
            self.len - 1
        }
    }
}

// TODO: Send a new event, Context (Surround?) with context data of the current selection
// TODO: OR bundle up the context on the selected candidate

/// Run the search engine task
pub async fn task(mut input_recv: Receiver<Event>, output_sender: Sender<Event>) -> Result<()> {
    log::trace!("starting search engine");

    let mut pool: VecDeque<Text> = VecDeque::new();
    let mut count = 0;
    let mut query = String::from("");
    let mut selection_ref = SelectionRef::new();

    while let Some(event) = input_recv.next().await {
        match event {
            Event::NewLine(s) => {
                log::trace!("line: {:?}", s);

                // Push the new line into the main pool
                pool.push_back(TextBuilder::build(&s));
                count += 1;

                // The pool might be full (too many lines in memory)
                // so we drop the first line
                if pool.len() > POOL_LIMIT {
                    log::trace!(
                        "pool limit ({:?}) exceeded, dropping first line",
                        POOL_LIMIT
                    );
                    let _f = pool.pop_front();
                }

                // We've got enough lines to refresh the search and send it
                // to the screen
                if count > BUFFER_LIMIT {
                    count = 0;
                    let matches = fuzzy::search(&query, &pool);
                    selection_ref.adjust(matches.len());
                    output_sender
                        .send(Event::Flush((matches, pool.len(), selection_ref.index())))
                        .await?;
                }
            }
            Event::EOF => {
                log::trace!("all input data done");
                let matches = fuzzy::search(&query, &pool);
                selection_ref.adjust(matches.len());
                output_sender
                    .send(Event::Flush((matches, pool.len(), selection_ref.index())))
                    .await?;
            }
            Event::Search(prompt) => {
                query = prompt.as_string();
                log::trace!("performing new search: '{}'", query);

                let matches = fuzzy::search(&query, &pool);
                selection_ref.adjust(matches.len());
                let results = Event::SearchDone((
                    matches,
                    pool.len(),
                    selection_ref.index(),
                    prompt.timestamp(),
                ));

                output_sender.send(results).await?;
            }
            Event::Up => {
                log::trace!("moving selection up");
                selection_ref.up();
                output_sender
                    .send(Event::Select(selection_ref.index()))
                    .await?;
            }
            Event::Down => {
                log::trace!("moving selection down");
                selection_ref.down();
                output_sender
                    .send(Event::Select(selection_ref.index()))
                    .await?;
            }
            Event::Done | Event::Exit => break,
            _ => (),
        };
    }

    log::trace!("search engine done");

    Ok(())
}
