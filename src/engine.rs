use crate::common::{Result, SearchBox, Text, TextBuilder};
use crate::config::Config;
use crate::events::Event;
use crate::fuzzy;
use async_std::prelude::*;
use async_std::sync::{Receiver, Sender};
use async_std::task::{Context, Poll};
use futures::stream::select;
use futures_timer::Delay;
use log::debug;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

// TODO: Move limits to Config
const BUFFER_LIMIT: usize = 5000;
const POOL_LIMIT: usize = 500000;

fn debounce(s: impl Stream<Item = Event> + Unpin) -> impl Stream<Item = Event> + Unpin {
    struct Debounce<S> {
        stream: S,
        delay: Delay,
        last: Option<SearchBox>,
    };

    impl<S: Stream<Item = Event> + Unpin> Stream for Debounce<S> {
        type Item = S::Item;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            if Pin::new(&mut self.delay).poll(cx).is_ready() {
                let mut result = None;
                if let Some(search_box) = self.last.take() {
                    result = Some(Poll::Ready(Some(Event::Search(search_box))));
                }

                if let Some(poll) = result {
                    return poll;
                }
            };

            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(event)) => {
                    match event {
                        Event::Search(search_box) => {
                            self.last = Some(search_box);
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

pub async fn task(
    _config: Config,
    input_recv: Receiver<Event>,
    output_sender: Sender<Event>,
) -> Result<()> {
    debug!("[task] start");

    let mut pool: VecDeque<Text> = VecDeque::new();
    let mut count = 0;
    let mut query = String::from("");

    // let mut incoming = debounce(input_recv);
    let mut incoming = input_recv;

    while let Some(event) = incoming.next().await {
        debug!("Got event {:?}", event);

        match event {
            Event::NewLine(s) => {
                pool.push_back(TextBuilder::build(&s));
                count += 1;

                if pool.len() > POOL_LIMIT {
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
                let matches = fuzzy::search(&query, &pool);
                output_sender
                    .send(Event::Flush((matches, pool.len())))
                    .await;
            }
            Event::Search(search_box) => {
                output_sender.send(Event::Search(search_box.clone())).await;

                query = search_box.as_string();
                let matches = fuzzy::search(&query, &pool);
                let results = Event::SearchDone((matches, pool.len(), search_box.timestamp()));

                output_sender.send(results).await;
            }
            Event::Done | Event::Exit => {
                output_sender.send(event).await;

                break;
            }
            _ => output_sender.send(event).await,
        };
    }

    drop(incoming);

    debug!("[task] end");

    Ok(())
}
