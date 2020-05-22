# Contributing

First of all, thank you for considering contributing to `scout`. This is a small project and it has a small set of features, but any contribution is always welcome.

Before contributing make sure you read the project's [Code of conduct][coc].

## How to contribute

Feel free to open an issue or a pull request explaining what you want to do or what bug did you find, for example.

Some things that you can do:

* Review a [Pull Request][pulls]
* Fix an [issue][issues]
* Report a bug
* Update or improve the [documentation][docs]
* Make a website

If you are going to report a bug these would be some good things to put in your report:

* Some reproduction steps
* OS where it happend
* How did you install the package (from source, from a package manager)
* Rust version

## Development

The code base runs against Rust stable. You'll need Rust `v1.43` or higher.

To install Rust it's better if you checkout [`rustup`][rustup].

If you make any change and you want to manually check the changes, you can use `cargo run`:

```
# use -- to pass options to the program
$ ls | cargo run -- --inline
```

You can run tests with the standard `cargo` command:

```
$ cargo test
```

### Linter

Use [`rustfmt`][rustfmt] as the default linter:

```
$ rustup compose add rustfmt
$ cargo fmt
```

[coc]: ./CODE_OF_CONDUCT.md
[pulls]: https://github.com/jhbabon/scout/pulls
[issues]: https://github.com/jhbabon/scout/issues
[docs]: ./README.md
[rustup]: https://rustup.rs/
