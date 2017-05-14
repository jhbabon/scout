# Scout

Scout is a small fuzzy finder for you terminal made with `rust`.

Yes, this is yet another tool inspired by [selecta]. The main difference with
[selecta], apart of the language, is the matching and scoring algorithm.

I decided to implement the matching algorithm with [regular expressions]. Call me
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
of it to compile and run the program.

Clone the repository and run `cargo install`. You can also run `cargo build` if
you want only to play with it:

```
$ git clone https://github.com/jhbabon/scout.git scout
$ cd scout
$ cargo install
```

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

### VIM integration

You can use the [selecta] vim snippets with `scout`, they pretty much work. A
fancier plugin to integrate `scout` with [neovim] is in the works, so stay
tunned!

[selecta]: https://github.com/garybernhardt/selecta
[regular expressions]: http://blog.amjith.com/fuzzyfinder-in-10-lines-of-python
[rust-stable]: https://www.rust-lang.org/downloads.html
[neovim]: https://neovim.io/
