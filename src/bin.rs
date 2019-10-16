#[macro_use]
extern crate log;

use rayon::prelude::*;

use std::process;

use async_std::prelude::*;
use async_std::future::join;
use async_std::task;
use async_std::os::unix::io::AsRawFd;

use futures::channel;

use scout::result::Result;
use scout::tty;
use scout::events::Event;
use scout::input;

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

// TODO: Move to lib
async fn broker_loop(mut events: Receiver<Event>) -> Result<Option<String>> {
    debug!("[broker_loop] start");

    // Get all outputs
    // NOTE: If we want to move the output to another task
    //   the State needs to implement Copy and that might be too much
    //   for this scenario (or not)
    let mut tty_out = tty::get_tty().await?;

    tty::into_raw_output(tty_out.as_raw_fd())?;

    let mut exit_event: Event = Event::Ignore;
    let mut state = State::new();

    while let Some(event) = events.next().await {
        match event {
            Event::Packet(s) => {
                state.add_string(s);
            },
            Event::Input(ch) => {
                state.add_char(ch);
                debug!("[broker_loop] start fuzzy search");
                state.search();
                debug!("[broker_loop] end fuzzy search");
            },
            Event::Done | Event::Exit => {
                exit_event = event;
                break
            },
            _ => (),
        };

        let l = format!("query: {:?}\nmatches: {:?}\n", state.query, state.matches);
        tty_out.write_all(l.as_bytes()).await?;
        tty_out.flush().await?;
    };

    debug!("[broker_loop] end");

    match exit_event {
        Event::Done => Ok(state.matches.pop()),
        _ => Ok(None),
    }
}

fn main() {
    env_logger::init();

    debug!("[main] start");

    let res = task::block_on(async {
        let (broker_sender, broker_receiver) = channel::mpsc::unbounded::<Event>();

        let broker = task::spawn(broker_loop(broker_receiver));
        let input = task::spawn(input::task(broker_sender));

        let (broker_result, input_result) = join!(broker, input).await;
        let _i = input_result?;

        broker_result
    });

    debug!("[main] end: {:?}", res);

    match res {
        Ok(Some(selection)) => println!("{}", selection),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            process::exit(1);
        }
        _ => (),
    }
}
