#![recursion_limit="256"]
// use futures::{select, FutureExt};
use async_std::io;
use async_std::os::unix::io::AsRawFd;
use async_std::prelude::*;
use async_std::task;
use async_std::fs;
use termios::{self, Termios};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn get_tty() -> Result<fs::File> {
    let tty = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .await?;

    Ok(tty)
}

fn main() -> Result<()> {
    task::block_on(async {
        // Get all inputs
        let stdin = io::stdin();
        let tty_in = get_tty().await?;

        // Get all outputs
        let mut tty_out = get_tty().await?;

        // TODO: This raw mode may need to be different than tty_out
        // BEGIN: RAW MODE
        let fd = tty_in.as_raw_fd();
        let tty = Termios::from_fd(fd)?;
        let mut raw_tty = tty;
        raw_tty.c_lflag &= !(termios::ICANON | termios::ECHO);
        termios::tcsetattr(fd, termios::TCSANOW, &raw_tty)?;
        // END:   RAW MODE

        // BEGIN: RAW MODE
        let fd = tty_out.as_raw_fd();
        let tty = Termios::from_fd(fd)?;
        let mut raw_tty = tty;
        raw_tty.c_lflag &= !(termios::ICANON | termios::ECHO);
        termios::tcsetattr(fd, termios::TCSANOW, &raw_tty)?;
        // END:   RAW MODE

        let std_reader = io::BufReader::new(stdin);
        let tty_reader = io::BufReader::new(tty_in);

        let std_lines = std_reader.lines();
        let tty_lines = tty_reader.lines();

        let mut all = futures::stream::select(tty_lines, std_lines);

        while let Some(line) = all.next().await {
            let l = format!("{}\n", line?);
            tty_out.write_all(l.as_bytes()).await?;
            tty_out.flush().await?;
        }

        Ok(())
    })
}
