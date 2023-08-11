#[macro_use]
extern crate log;

use async_std::io;
use async_std::os::unix::io::AsRawFd;
use async_std::task;
use std::convert::TryFrom;
use std::process;

use scout::common::{Result, Text};
use scout::config::{Args, Configurator};
use scout::ptty::{self, PTTY};
use scout::supervisor;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const HELP: &str = r#"Your friendly fuzzy finder

USAGE:
    scout [FLAGS] [OPTIONS]

FLAGS:
    -f, --full-screen    Show scout in full screen (default)
    -h, --help           Prints help information
    -i, --inline         Show scout under the current line
    -n, --no-sort        Do not sort the result by score
    -v, --version        Prints version information

OPTIONS:
    -c, --config <FILE>     Uses a custom config file
    -l, --lines <LINES>     Number of lines to display in inline mode, including prompt
    -s, --search <QUERY>    Start searching with the given query
    -p, --pool <SIZE>       Advanced: size of the pool of candidates to keep in memory.
                            Default is 50000. Note that increasing this number might
                            result in the program using too much memory

SUPPORTED KEYS:
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
    $ git branch --sort=-committerdate| cut -c 3- | scout -i -n | xargs git checkout"#;

fn main() {
    env_logger::init();

    trace!("starting main program");

    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}.", e);
            process::exit(3);
        }
    };

    trace!("got args: {:?}", args);

    let res: Result<Option<Text>> = task::block_on(async {
        let mut configurator = Configurator::new();

        match &args.config {
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

fn parse_args() -> std::result::Result<Args, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        println!("scout {}", VERSION.unwrap_or("unknown"));
        println!("{}", HELP);
        process::exit(0);
    }

    if pargs.contains(["-v", "--version"]) {
        println!("scout {}", VERSION.unwrap_or("unknown"));
        process::exit(0);
    }

    let search = match pargs.opt_value_from_str(["-s", "--search"]) {
        Ok(s) => s,
        Err(e) => {
            match e {
                // Allow empty string arguments like --search="" or -s ""
                pico_args::Error::OptionWithoutAValue(_) => {
                    // Ensure the flags -s= and --search= are removed from the list
                    let _ = pargs.contains(["-s=", "--search="]);
                    None
                }
                _ => return Err(e),
            }
        }
    };

    let args = Args {
        // flags
        full_screen: pargs.contains(["-f", "--full-screen"]),
        inline: pargs.contains(["-i", "--inline"]),
        no_sort: pargs.contains(["-n", "--no-sort"]),

        // options
        search,
        lines: pargs.opt_value_from_str(["-l", "--lines"])?,
        config: pargs.opt_value_from_str(["-c", "--config"])?,
        pool: pargs.opt_value_from_str(["-p", "--pool"])?,
    };

    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Error: unknown command line arguments: {:?}.", remaining);
        process::exit(2);
    }

    Ok(args)
}
