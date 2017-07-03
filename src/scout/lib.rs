extern crate libc;
extern crate termios;
extern crate termion;
extern crate regex;
extern crate num_cpus;
extern crate futures;
extern crate futures_cpupool;

mod score;
mod choice;
mod pattern;
mod terminal_size;
mod terminal;
mod scout;
mod refine;

pub mod ui;
pub mod errors;
pub use choice::Choice;
pub use terminal::Terminal;
pub use scout::Scout;
pub use refine::refine;

/// Get the version of the program.
pub fn version() -> String {
    let (maj, min, pat) = (
        option_env!("CARGO_PKG_VERSION_MAJOR"),
        option_env!("CARGO_PKG_VERSION_MINOR"),
        option_env!("CARGO_PKG_VERSION_PATCH"),
    );

    match (maj, min, pat) {
        (Some(maj), Some(min), Some(pat)) => format!("{}.{}.{}", maj, min, pat),
        _ => "".to_string(),
    }
}
