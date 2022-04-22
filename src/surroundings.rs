use crate::common::{Pool, Result, Text};
use crate::events::Event;

use async_std::channel::{Receiver, Sender};
use async_std::prelude::*;

/// This task looks for the surrounding text around a given candidate and returns it, all through
/// events.
pub async fn task(mut recv: Receiver<Event>, sender: Sender<Event>) -> Result<()> {
    let mut pool: Option<Pool<Text>> = None;

    while let Some(event) = recv.next().await {
        match event {
            Event::Pool(pl) => pool = Some(pl),
            Event::Surroundings(candidate) => {
                if let Some(lock) = &pool {
                    let idx = candidate.pool_idx();
                    // TODO: Iterate over a range on each direction
                    let (before_idx, overflow_before) = idx.overflowing_sub(1);
                    let (after_idx, overflow_after) = idx.overflowing_add(1);

                    let mut before: Vec<Text> = vec![];
                    let mut after: Vec<Text> = vec![];

                    let pl = lock.read().await;

                    if !overflow_before {
                        before.push(pl[before_idx].clone());
                    }

                    if !overflow_after && after_idx < pl.len() {
                        after.push(pl[after_idx].clone());
                    }

                    sender
                        .send(Event::SurroundingsDone((before, after)))
                        .await?;
                }
            }
            Event::Done | Event::Exit => break,
            _ => (),
        }
    }
    Ok(())
}
