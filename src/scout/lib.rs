mod score;
mod choice;
mod pattern;
mod explore;
mod terminal;

pub mod ui;
pub use explore::explore;
pub use terminal::Terminal;

/// Get the version of the program.
pub fn version() -> String {
    let (maj, min, pat) = (
        option_env!("CARGO_PKG_VERSION_MAJOR"),
        option_env!("CARGO_PKG_VERSION_MINOR"),
        option_env!("CARGO_PKG_VERSION_PATCH")
    );

    match (maj, min, pat) {
        (Some(maj), Some(min), Some(pat)) => {
            format!("{}.{}.{}", maj, min, pat)
        },
        _ => "".to_string(),
    }
}
