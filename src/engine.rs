use log::debug;
use std::collections::VecDeque;
use rayon::prelude::*;
use async_std::prelude::*;
use futures::stream::select;
use futures::SinkExt;
use futures::channel::mpsc::{Receiver,Sender};
use crate::config::Config;
use crate::common::Result;
use crate::events::Event;
use crate::fuzzy::{self,Candidate};

const BUFFER_LIMIT: usize = 5000;
const POOL_LIMIT: usize = 500000;

pub async fn task(_config: Config, pipe: Receiver<Event>, input: Receiver<Event>, mut output: Sender<Event>) -> Result<()> {
    debug!("[task] start");

    let mut pool: VecDeque<Candidate> = VecDeque::new();
    let mut count = 0;
    let mut query = String::from("");

    let mut incoming = select(input, pipe);

    while let Some(event) = incoming.next().await {
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
            },
            Event::EOF => {
                let matches = search(&query, &pool);

                Some(Event::FlushSearch((matches, pool.len())))
            },
            Event::Query((q, ts)) => {
                query = q;
                let matches = search(&query, &pool);

                Some(Event::Search((matches, pool.len(), ts)))
            },
            Event::Done | Event::Exit => {
                break
            },
            _ => None,
        };

        if let Some(forward) = next {
            output.send(forward).await?
        }
    };

    drop(output);
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
