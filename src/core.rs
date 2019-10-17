use log::debug;
use rayon::prelude::*;
use async_std::prelude::*;
use futures::channel;
use crate::result::Result;
use crate::ptty::get_ptty;
use crate::events::Event;

type Receiver<T> = channel::mpsc::UnboundedReceiver<T>;

#[derive(Debug,Clone, Default)]
struct State {
    pub query: Vec<char>,
    pub pool: Vec<String>,
    pub matches: Vec<String>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_char(&mut self, ch: char) {
        self.query.push(ch);
    }

    pub fn add_string(&mut self, string: String) {
        self.pool.push(string);
    }

    // NOTE: This is just temporary, the search should
    // be outside the state
    pub fn search(&mut self) {
        let q = self.query.iter().collect::<String>();

        self.matches = self.pool
            .par_iter()
            .filter(|s| s.contains(q.as_str()))
            .map(|s| s.clone())
            .collect();
    }
}

pub async fn task(mut events: Receiver<Event>) -> Result<Option<String>> {
    debug!("[task] start");

    // Get all outputs
    // NOTE: If we want to move the output to another task
    //   the State needs to implement Copy and that might be too much
    //   for this scenario (or not)
    let mut ptty_out = get_ptty().await?;

    let mut exit_event: Event = Event::Ignore;
    let mut state = State::new();

    while let Some(event) = events.next().await {
        match event {
            Event::Packet(s) => {
                state.add_string(s);
            },
            Event::Input(ch) => {
                state.add_char(ch);
                debug!("[task] start fuzzy search");
                state.search();
                debug!("[task] end fuzzy search");
            },
            Event::Done | Event::Exit => {
                exit_event = event;
                break
            },
            _ => (),
        };

        let l = format!("query: {:?}\nmatches: {:?}\n", state.query, state.matches);
        ptty_out.write_all(l.as_bytes()).await?;
        ptty_out.flush().await?;
    };

    debug!("[task] end");

    match exit_event {
        Event::Done => Ok(state.matches.pop()),
        _ => Ok(None),
    }
}
