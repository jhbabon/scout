# Scout

Scout is a small fuzzy finder for you terminal made with `rust`.

Yes, this is yet another tool inspired by [selecta]. The main difference with
[selecta][], apart of the language, is the matching and scoring algorithm.

I decided to implement the matching algorithm with [regular expressions][]. Call me
crazy, but life is too sort to iterate over endless strings keeping track of
indexes of chars with variable sizes. Ok, maybe I'm just bad doing those kind of
algorithms. Also, I like regexes, they are kind of a drug for me.

## WARNING

I consider this a beta. There are a lot of tests missing and, to be
honest, there are parts of `rust` that I don't understand and I basically copy
and paste things from around, so there is a lot of room for improvement.

Scout has been only tested agains linux. It probably works against macOS or
any other UNIX as well, but it is not intended to work on Windows.

## Installation

Scout is made with `rust`, so you will need the [latest stable version][rust-stable]
of it to compile and run the program. Check out [rustup][] for `rust`
installations.

### Install via cargo

Scout is in the main [crates][] repository, so you can install it just with `cargo`:

```
$ cargo install scout
```

Remember to put `cargo` bin path to the main `$PATH` env variable:

```
export PATH=$PATH:~/.cargo/bin
```

### Install via git

Clone the repository and run `cargo install` from it:

```
$ git clone https://github.com/jhbabon/scout.git path/to/scout
$ cd path/to/scout
$ cargo install
```

You can also run `cargo build --release` if you just want to play with it.

## Usage

The main idea is to use this tool with pipes. You get a list of items that you
want to filter and pass it to `scout`. Once you select the item you want,
`scout` will print it to the standard output (stdout).

[![asciicast](https://asciinema.org/a/120469.png)](https://asciinema.org/a/120469)

You can always check the `--help` option for more info:

```
$ scout --help
Scout: Small fuzzy finder

This program expects a list of items in the standard input,
so it is better to use it with pipes.

Usage:
  scout [options]

Options:
  -h --help     Show this screen.
  -v --version  Show version.

Supported keys:
   * ^U to delete the entire line
   * ^N or Arrow key down to select the next match
   * ^P or Arrow key up to select the previous match
   * ESC to quit without selecting a match

Example:
  $ ls | scout
```

### NEOVIM integration

I made a plugin to use `scout` inside [neovim][] thanks to its built in
`:terminal` emulator. It's called [scout.vim][].

## Development

Scout compiles against the [latest stable `rust` version][rust-stable],
so if you want to hack with it be sure to use it.

There are (some) tests. You can run them with `cargo test`:

```
$ cargo test
```

### Code formatting

Use [rustfmt][] to format the code in a consistent manner. More precisely, use
`rustfmt-nightly`. Since `scout` is built against the stable version of `rust`,
you can use `rustfmt-nightly` with [rustup][]:

```
# Install rustfmt-nightly
$ rustup run nightly cargo install rustfmt-nightly

# Run it with cargo
$ rustup run nightly cargo fmt
```

[selecta]: https://github.com/garybernhardt/selecta
[regular expressions]: http://blog.amjith.com/fuzzyfinder-in-10-lines-of-python
[rust-stable]: https://www.rust-lang.org/downloads.html
[crates]: https://crates.io/crates/scout
[neovim]: https://neovim.io/
[rustfmt]: https://github.com/rust-lang-nursery/rustfmt
[rustup]: https://www.rustup.rs/
[scout.vim]: https://github.com/jhbabon/scout.vim
