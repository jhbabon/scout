#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use async_std::io;
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

    trace!("starting main program");

    let args = App::new("scout")
        .version(crate_version!())
        .arg(
            Arg::with_name("inline")
                .short("i")
                .long("inline")
                .help("Show fuzzy finder under the current line"),
        )
        .arg(
            Arg::with_name("full-screen")
                .short("f")
                .long("full-screen")
                .help("Show fuzzy finder in full screen (default)"),
        )
        .arg(
            Arg::with_name("lines")
                .short("l")
                .long("lines")
                .value_name("LINES")
                .takes_value(true)
                .help("Number of lines to display in inline mode, including prompt"),
        )
        .arg(
            Arg::with_name("search")
                .short("s")
                .long("search")
                .value_name("QUERY")
                .takes_value(true)
                .help("Start searching with the given query"),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .takes_value(true)
                .help("Sets a custom config file"),
        )
        .get_matches();

    trace!("got args: {:?}", args);

    let res: Result<Option<Text>> = task::block_on(async {
        let tty = get_ptty().await?;

        let mut configurator = Configurator::new();

        match args.value_of("config") {
            Some(config_path) => configurator.from_file(config_path),
            None => configurator.from_default_file(),
        };

        let config = configurator.from_ptty(&tty).from_args(&args).build();

        trace!("generated config: {:?}", config);

        // We only need to set up the ptty into noncanonical mode once
        let ptty = PTTY::try_from(tty.as_raw_fd())?;
        ptty.noncanonical_mode()?;

        let stdin = io::stdin();
        let pttyin = get_ptty().await?;
        let pttyout = get_ptty().await?;

        supervisor::run(config, stdin, pttyin, pttyout).await
    });

    trace!("program ended with {:?}", res);

    match res {
        Ok(Some(selection)) => println!("{}", selection),
        Ok(None) => process::exit(130),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            process::exit(1);
        }
    }
}
