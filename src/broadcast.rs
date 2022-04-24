// FIXME: Docs!
use crate::common::Result;
use crate::events::Event;

use async_std::channel::{self, Receiver, Sender};
use std::collections::HashMap;

// Tasks that can receive events
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum Task {
    Engine,
    Screen,
    Surroundings,
}

#[derive(Clone)]
pub struct Broadcaster {
    senders: HashMap<Task, Sender<Event>>,
}

impl Broadcaster {
    pub fn new(senders: HashMap<Task, Sender<Event>>) -> Self {
        Self { senders }
    }

    pub fn on(&self, tasks: &[Task]) -> Result<Self> {
        let mut senders = HashMap::new();

        for task in tasks {
            let sender = self
                .senders
                .get(task)
                .ok_or_else(|| format!("Task '{:?}' not found", task))?;
            senders.insert(task.clone(), sender.clone());
        }

        Ok(Self::new(senders))
    }

    pub async fn send_to(&self, event: Event, task: Task) -> Result<()> {
        let sender = self
            .senders
            .get(&task)
            .ok_or_else(|| format!("Task '{:?}' not found", task))?;
        sender.send(event).await?;

        Ok(())
    }

    pub async fn send_many(&self, event: Event, tasks: &[Task]) -> Result<()> {
        let len = tasks.len();
        let mut last = None;

        for (index, task) in tasks.iter().enumerate() {
            let sender = self
                .senders
                .get(task)
                .ok_or_else(|| format!("Task '{:?}' not found", task))?;
            if (index + 1) < len {
                sender.send(event.clone()).await?;
            } else {
                last = Some(sender);
            }
        }

        if let Some(sender) = last {
            // Do not clone last event sent
            sender.send(event).await?;
        }

        Ok(())
    }

    pub async fn send_all(&self, event: Event) -> Result<()> {
        let values = self.senders.values();
        let len = values.len();
        let mut last = None;

        for (index, sender) in values.enumerate() {
            if (index + 1) < len {
                sender.send(event.clone()).await?;
            } else {
                last = Some(sender);
            }
        }

        if let Some(sender) = last {
            // Do not clone last event sent
            sender.send(event).await?;
        }

        Ok(())
    }
}

pub struct Antenna {
    receivers: HashMap<Task, Receiver<Event>>,
}

impl Antenna {
    pub fn new(receivers: HashMap<Task, Receiver<Event>>) -> Self {
        Self { receivers }
    }

    pub fn on(&self, task: Task) -> Result<Receiver<Event>> {
        let receiver = self
            .receivers
            .get(&task)
            .ok_or_else(|| format!("Task '{:?}' not found", task))?;

        Ok(receiver.clone())
    }
}

pub fn broadband(size: usize, tasks: &[Task]) -> (Broadcaster, Antenna) {
    let mut senders = HashMap::new();
    let mut receivers = HashMap::new();

    for task in tasks {
        let (sender, receiver) = channel::bounded::<Event>(size);
        senders.insert(task.clone(), sender);
        receivers.insert(task.clone(), receiver);
    }

    (Broadcaster::new(senders), Antenna::new(receivers))
}
