use log::debug;
use std::collections::VecDeque;
use rayon::prelude::*;
use async_std::prelude::*;
use futures::stream::select;
use futures::SinkExt;
use futures::channel::mpsc::{Receiver,Sender};
use crate::result::Result;
use crate::events::Event;
use crate::fuzzy::Candidate;

const BUFFER_LIMIT: usize = 5000;
const POOL_LIMIT: usize = 100000;

pub async fn task(pipe: Receiver<Event>, input: Receiver<Event>, mut screen: Sender<Event>) -> Result<()> {
    debug!("[task] start");

    let mut should_search: bool;

    let mut pool: VecDeque<Candidate> = VecDeque::new();
    let mut count = 0;
    let mut query = String::from("");

    let mut incoming = select(input, pipe);

    while let Some(event) = incoming.next().await {
        should_search = false;

        match event {
            Event::Packet(s) => {
                pool.push_back(Candidate::new(s));
                count += 1;

                if pool.len() > POOL_LIMIT {
                    let _f = pool.pop_front();
                }

                if count > BUFFER_LIMIT {
                    count = 0;
                    should_search = true;
                }
            },
            Event::EOF => {
                should_search = true;
            },
            Event::Query(q) => {
                query = q;
                should_search = true;
            },
            Event::Done | Event::Exit => {
                break
            },
            _ => (),
        };

        if should_search {
            debug!("[task|while] searching with: {:?}", query);
            let mut matches: Vec<Candidate>;

            instrument!("search", {
                if query.is_empty() {
                    matches = pool.par_iter().cloned().collect();
                } else {
                    matches = pool
                        .par_iter()
                        .map(|c| Candidate::best_match(&query, &c.string))
                        .filter(|c| c.is_some())
                        .map(|c| c.unwrap())
                        .collect();

                    matches.par_sort_unstable_by(|a, b| b.cmp(a));
                }
            });

            screen.send(Event::Matches(matches)).await?;
        }
    };

    drop(screen);
    drop(incoming);

    debug!("[task] end");

    Ok(())
}
