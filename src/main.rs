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
            Arg::with_name("full-screen")
                .short("f")
                .long("full-screen")
                .help("Show fuzzy finder in full screen (default)"),
        )
        .arg(
            Arg::with_name("inline")
                .short("i")
                .long("inline")
                .help("Show fuzzy finder under the current line"),
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
        let mut configurator = Configurator::new();

        match args.value_of("config") {
            Some(config_path) => configurator.from_file(config_path),
            None => configurator.from_default_file(),
        };

        // PTTY = Pseudo Terminal
        let tty = get_ptty().await?;
        let config = configurator.from_ptty(&tty).from_args(&args).build();

        trace!("generated config: {:?}", config);

        // We only need to set up the ptty into noncanonical mode once
        let ptty = PTTY::try_from(tty.as_raw_fd())?;
        ptty.noncanonical_mode()?;

        // Get the list of candidates to filter from the STDIN
        // This list comes most probably from a pipe
        let stdin = io::stdin();

        // The architecture of the app is async, one task (a future) will
        // read from the PTTY while another task will write to it. We can't
        // have one single mutable PTTY reference across these task because
        // locking to read would mean blocking the screen when printing.
        // To overcome this we create two different references to the
        // system's PTTY, one for reading and one for writting.
        //
        // Even though the path to the PTTY is the same, reading from and
        // writting to it are fundamentally two different and independent steps.
        //
        // Is this a hack? Most probably. Does it work? Yes.
        let pttyin = get_ptty().await?; // to read person's input
        let pttyout = get_ptty().await?; // to print programs interface

        // The main program's thread will block until the supervisor's task finishes
        // thanks to the `task::block_on` call
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
