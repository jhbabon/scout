//! Search engine: Where the fuzzy matching magic happens
//!
//! This task will collect all the input from STDIN and search over them on new queries.
//! Once a search is done all the results will be sent to the screen.

use crate::common::{Prompt, Result, Text, TextBuilder};
use crate::events::Event;
use crate::fuzzy;
use async_std::prelude::*;
use async_std::sync::{Receiver, Sender};
use async_std::task::{Context, Poll};
use futures_timer::Delay;
use log;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

// TODO: Move limits to Config
const BUFFER_LIMIT: usize = 5000;
const POOL_LIMIT: usize = 500000;

/// Run the search engine task
pub async fn task(input_recv: Receiver<Event>, output_sender: Sender<Event>) -> Result<()> {
    log::trace!("starting search engine");

    let mut pool: VecDeque<Text> = VecDeque::new();
    let mut count = 0;
    let mut query = String::from("");
    let mut incoming = debounce(input_recv);

    while let Some(event) = incoming.next().await {
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
                    output_sender
                        .send(Event::Flush((matches, pool.len())))
                        .await;
                }
            }
            Event::EOF => {
                log::trace!("all input data done");
                let matches = fuzzy::search(&query, &pool);
                output_sender
                    .send(Event::Flush((matches, pool.len())))
                    .await;
            }
            Event::Search(prompt) => {
                query = prompt.as_string();
                log::trace!("performing new search: '{}'", query);

                let matches = fuzzy::search(&query, &pool);
                let results = Event::SearchDone((matches, pool.len(), prompt.timestamp()));

                output_sender.send(results).await;
            }
            Event::Done | Event::Exit => break,
            _ => (),
        };
    }

    log::trace!("search engine done");

    Ok(())
}

/// Sometimes we type two or three characters in the prompt very fast. In that case we don't expect
/// the program to search over each one of the characters but only over all of them.
/// This function tries to prevent that scenario where each character performs a search.
///
/// Whenever a new character is added to the search query, this stream waits a little bit to see
/// if a new character arrives. If a character arrives before the time limit, it will send the
/// search with the new character ignoring the previous incomplete search request.
///
/// Note that given this is time based, and async, it's not exact and some intermediate search will
/// go through. It really depends on how fast you type. Let's say it's good enough.
fn debounce(stream: impl Stream<Item = Event> + Unpin) -> impl Stream<Item = Event> + Unpin {
    struct Debounce<S> {
        stream: S,
        delay: Delay,
        is_delayed: bool,
        last: Option<Prompt>,
    };

    impl<S: Stream<Item = Event> + Unpin> Stream for Debounce<S> {
        type Item = S::Item;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            // If we reached the time limit, try to send the accumulated search request
            if Pin::new(&mut self.delay).poll(cx).is_ready() {
                self.is_delayed = false;
                if let Some(prompt) = self.last.take() {
                    return Poll::Ready(Some(Event::Search(prompt)));
                }
            };

            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(event)) => {
                    match event {
                        Event::Search(prompt) => {
                            self.last = Some(prompt);
                            if !self.is_delayed {
                                // Only reset the delay if it wasn't delayed
                                // before. If it was delayed (a new search query started)
                                // then wait for the delay to finish before resetting it
                                self.delay.reset(Duration::from_millis(150));
                                self.is_delayed = true;
                            }

                            // We have to return Ready if we want to collect all
                            // the events from the original stream
                            Poll::Ready(Some(Event::Ignore))
                        }
                        _ => Poll::Ready(Some(event)),
                    }
                }
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            }
        }
    }

    Debounce {
        stream,
        delay: Delay::new(Duration::from_secs(0)),
        is_delayed: false,
        last: Default::default(),
    }
}
