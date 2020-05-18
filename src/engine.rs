use crate::common::{Prompt, Result, Text, TextBuilder};
use crate::events::Event;
use crate::fuzzy;
use async_std::prelude::*;
use async_std::sync::{Receiver, Sender};
use async_std::task::{Context, Poll};
use futures::stream::select;
use futures_timer::Delay;
use log;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

// TODO: Move limits to Config
const BUFFER_LIMIT: usize = 5000;
const POOL_LIMIT: usize = 500000;

pub async fn task(input_recv: Receiver<Event>, output_sender: Sender<Event>) -> Result<()> {
    log::trace!("starting search engine");

    let mut pool: VecDeque<Text> = VecDeque::new();
    let mut count = 0;
    let mut query = String::from("");

    // let mut incoming = debounce(input_recv);
    let mut incoming = input_recv;

    while let Some(event) = incoming.next().await {
        match event {
            Event::NewLine(s) => {
                log::trace!("line: {:?}", s);

                pool.push_back(TextBuilder::build(&s));
                count += 1;

                if pool.len() > POOL_LIMIT {
                    log::trace!(
                        "pool limit ({:?}) exceeded, dropping first line",
                        POOL_LIMIT
                    );
                    let _f = pool.pop_front();
                }

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

fn debounce(s: impl Stream<Item = Event> + Unpin) -> impl Stream<Item = Event> + Unpin {
    struct Debounce<S> {
        stream: S,
        delay: Delay,
        last: Option<Prompt>,
    };

    impl<S: Stream<Item = Event> + Unpin> Stream for Debounce<S> {
        type Item = S::Item;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            if Pin::new(&mut self.delay).poll(cx).is_ready() {
                let mut result = None;
                if let Some(prompt) = self.last.take() {
                    result = Some(Poll::Ready(Some(Event::Search(prompt))));
                }

                if let Some(poll) = result {
                    return poll;
                }
            };

            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(event)) => {
                    match event {
                        Event::Search(prompt) => {
                            self.last = Some(prompt);
                            // TODO: tune up the time
                            self.delay.reset(Duration::from_millis(200));

                            // We have to return Ready if we want to collect all
                            // the query events from the original stream
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
        stream: s,
        delay: Delay::new(Duration::from_secs(0)),
        last: Default::default(),
    }
}
