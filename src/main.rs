#[macro_use]
extern crate log;

use std::convert::TryFrom;
use std::process;
use async_std::io;
use async_std::task;
use async_std::os::unix::io::AsRawFd;

use scout::result::Result;
use scout::fuzzy::Text;
use scout::ptty::{get_ptty, PTTY};
use scout::supervisor;

fn main() {
    env_logger::init();

    debug!("[main] start");

    let res: Result<Option<Text>> = task::block_on(async {
        // We only need to set up the ptty into noncanonical mode once
        let tty = get_ptty().await?;
        let ptty = PTTY::try_from(tty.as_raw_fd())?;
        ptty.noncanonical_mode()?;

        let stdin = io::stdin();
        let pttyin = get_ptty().await?;
        let pttyout = get_ptty().await?;

        supervisor::run(stdin, pttyin, pttyout).await
    });

    debug!("[main] end: {:?}", res);

    match res {
        Ok(Some(selection)) => println!("{}", selection),
        Ok(None) => {
            process::exit(130);
        },
        Err(e) => {
            eprintln!("ERROR: {}", e);
            process::exit(1);
        }
    }
}
