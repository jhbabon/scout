use crate::common::Result;
use crate::config::Config;
use crate::events::Event;
use crate::fuzzy::{self, Candidate};
use async_std::prelude::*;
use async_std::sync::{Receiver, Sender};
use futures::stream::select;
use log::debug;
use rayon::prelude::*;
use std::collections::VecDeque;

const BUFFER_LIMIT: usize = 5000;
const POOL_LIMIT: usize = 500000;

pub async fn task(
    _config: Config,
    pipe_recv: Receiver<Event>,
    input_recv: Receiver<Event>,
    conveyor_sender: Sender<Event>,
) -> Result<()> {
    debug!("[task] start");

    let mut pool: VecDeque<Candidate> = VecDeque::new();
    let mut count = 0;
    let mut query = String::from("");

    let mut incoming = select(input_recv, pipe_recv);

    while let Some(event) = incoming.next().await {
        debug!("Got event {:?}", event);

        let next = match event {
            Event::Packet(s) => {
                pool.push_back(Candidate::new(s));
                count += 1;

                if pool.len() > POOL_LIMIT {
                    let _f = pool.pop_front();
                }

                if count > BUFFER_LIMIT {
                    count = 0;
                    let matches = search(&query, &pool);
                    Some(Event::FlushSearch((matches, pool.len())))
                } else {
                    None
                }
            }
            Event::EOF => {
                let matches = search(&query, &pool);

                Some(Event::FlushSearch((matches, pool.len())))
            }
            Event::Query((q, ts)) => {
                query = q;
                let matches = search(&query, &pool);

                Some(Event::Search((matches, pool.len(), ts)))
            }
            Event::Done | Event::Exit => break,
            _ => None,
        };

        if let Some(forward) = next {
            conveyor_sender.send(forward).await
        }
    }

    drop(conveyor_sender);
    drop(incoming);

    debug!("[task] end");

    Ok(())
}

fn search(query: &str, pool: &VecDeque<Candidate>) -> Vec<Candidate> {
    let mut matches: Vec<Candidate>;

    if query.is_empty() {
        matches = pool.par_iter().cloned().collect();
    } else {
        matches = pool
            .par_iter()
            .map(|c| fuzzy::finder(&query, c.text.clone()))
            .filter(|c| c.is_some())
            .map(|c| c.unwrap())
            .collect();

        matches.par_sort_unstable_by(|a, b| b.cmp(a));
    }

    matches
}
