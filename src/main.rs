#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use async_std::fs;
use async_std::io;
use async_std::io::prelude::*;
use async_std::os::unix::io::AsRawFd;
use async_std::task;
use clap::{App, Arg};
use std::convert::TryFrom;
use std::process;

use scout::common::{Result, Text};
use scout::config::Configurator;
use scout::ptty::{get_ptty, PTTY};
use scout::supervisor;

fn main() {
    env_logger::init();

    debug!("[main] start");

    let args = App::new("scout")
        .version(crate_version!())
        .arg(
            Arg::with_name("inline")
                .short("i")
                .long("inline")
                .help("show finder under the current line"),
        )
        .arg(
            Arg::with_name("lines")
                .short("l")
                .long("lines")
                .takes_value(true)
                .help("Number of lines to display in inline mode, including prompt"),
        )
        .arg(
            Arg::with_name("search")
                .short("s")
                .long("search")
                .takes_value(true)
                .help("Initial search"),
        )
        .get_matches();

    debug!("args: {:?}", args);

    let res: Result<Option<Text>> = task::block_on(async {
        // We only need to set up the ptty into noncanonical mode once
        let tty = get_ptty().await?;

        let mut configurator = Configurator::new();

        // TODO: Use $HOME/.scout.toml (or similar) path or pass it through args
        if let Ok(mut config_file) = fs::File::open("./config.toml").await {
            let mut contents = String::new();
            config_file.read_to_string(&mut contents).await?;
            configurator.from_toml(&contents);
        }

        let config = configurator.from_ptty(&tty).from_args(&args).build();

        debug!("config: {:?}", config);

        let ptty = PTTY::try_from(tty.as_raw_fd())?;
        ptty.noncanonical_mode()?;

        let stdin = io::stdin();
        let pttyin = get_ptty().await?;
        let pttyout = get_ptty().await?;

        supervisor::run(config, stdin, pttyin, pttyout).await
    });

    debug!("[main] end: {:?}", res);

    match res {
        Ok(Some(selection)) => println!("{}", selection),
        Ok(None) => process::exit(130),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            process::exit(1);
        }
    }
}
