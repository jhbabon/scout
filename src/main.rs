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
use scout::ptty::{self, PTTY};
use scout::supervisor;

const EXTENDED_HELP: &str = r#"SUPPORTED KEYS:
    - Enter to select the current highlighted match and print it to STDOUT
    - ^u to clear the prompt
    - ^n or Down arrow key to select the next match
    - ^p or Up arrow key to select the previous match
    - ^e to go to the end of the prompt
    - ^a to go to the beginning of the prompt
    - Left arrow key to move the cursor to the left in the prompt
    - Right arrow key to move the cursor to the right in the prompt
    - ESC to quit without selecting a match

EXAMPLES:
    $ find * -type f | scout

    # Pass an initial query to start filtering right away
    $ find * -type f | scout --search=foo

    # Use a custom config file
    $ find * -type f | scout --config="./config.toml"

    # Select a git branch and check it out with an inline menu
    $ git branch | cut -c 3- | scout -i | xargs git checkout"#;

fn main() {
    env_logger::init();

    trace!("starting main program");

    let args = App::new("scout")
        .version(crate_version!())
        .about("Your friendly fuzzy finder")
        .after_help(EXTENDED_HELP)
        .arg(
            Arg::with_name("full-screen")
                .short("f")
                .long("full-screen")
                .help("Show scout in full screen (default)"),
        )
        .arg(
            Arg::with_name("inline")
                .short("i")
                .long("inline")
                .help("Show scout under the current line"),
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
                .help("Uses a custom config file"),
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
        let tty = ptty::file().await?;
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
        // have one single mutable PTTY reference across these tasks because
        // locking to read would mean blocking printing to the screen.
        // To overcome this we create two different references to the
        // system's PTTY, one for reading and one for writting.
        //
        // Even though the path to the PTTY is the same, reading from and
        // writting to it are fundamentally two different and independent steps.
        //
        // Is this a hack? Most probably, yes. Does it work? Also yes.
        let pttyin = ptty::reader().await?; // to read person's input
        let pttyout = ptty::writer().await?; // to print programs interface

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
