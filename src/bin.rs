#[macro_use]
extern crate log;

use std::convert::TryFrom;
use std::process;
use async_std::future::join;
use async_std::task;
use async_std::os::unix::io::AsRawFd;
use futures::channel;
use scout::ptty::{PTTY, get_ptty};
use scout::events::Event;
use scout::interactions;
use scout::input;
use scout::core;

fn main() {
    env_logger::init();

    debug!("[main] start");

    let res = task::block_on(async {
        // We only need to set up the ptty into noncanonical mode once
        let tty = get_ptty().await?;
        let ptty = PTTY::try_from(tty.as_raw_fd())?;
        ptty.noncanonical_mode()?;

        // NOTE: Using a bounded channel helps when there
        // are too many incoming messages
        let (input_sender, input_receiver) = channel::mpsc::channel::<Event>(2046);
        let (int_sender, int_receiver) = channel::mpsc::channel::<Event>(25);

        let core = task::spawn(core::task(int_receiver, input_receiver));
        let interactions = task::spawn(interactions::task(int_sender));
        let input = task::spawn(input::task(input_sender));

        let (core_result, input_result, interactions_result) = join!(core, input, interactions).await;
        let _i = input_result?;
        let _i = interactions_result?;

        core_result
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
