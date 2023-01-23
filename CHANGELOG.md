# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- New argument, `--pool`, and config option `advanced.pool_size` (or `advanced.pool`).
  This option can change the size of the pool of candidates that are kept in memory.
  Increasing this number might result in the program using too much memory.
  This is an advanced feature and should be used with care.

### Changed
- Bump `unicode-segmentation` 1.9.0 to 1.10.0
- Bump `futures` from 0.3.21 to 0.3.25
- Bump `env_logger` from 0.9.0 to 0.10.0
- Bump `libc` from 0.2.129 to 0.2.139

### Fixed
- If the setting `screen.lines` is present in the TOML config file, apply it only on inline mode
- Respect `screen.lines` option from config file when no `--lines` argument is given
- Ensure `-v` flag is in lowercase in CLI help message

## [v2.6.0] 2022-01-23
### Changed
- Update to edition 2021 and set rust-version to 1.58
- Bump `serde` from 1.0.130 to 1.0.133
- Bump `futures` from 0.3.17 to 0.3.19
- Bump `libc` from 0.2.103 to 0.2.113

## [v2.5.1] 2021-09-21
### Fixed
Update crossbeam yanked dependencies:
- `crossbeam-deque` from `v0.8.0` to `v0.8.1`
- `crossbeam-epoch` from `v0.9.3` to `v0.9.5`
- `crossbeam-utils` from `v0.8.3` to `v0.8.5`

## [v2.5.0] 2021-09-21
### Changed
- Update `futures` from `v0.3.13` to `v0.3.17`
- Update `async-std` from `v1.9.0` to `v1.10.0`
- Update `dirs` from `v3.0.1` to `v4.0.0`
- Update `env_logger` from `v0.8.3` to `v0.9.0`
- Update `libc` from `v0.2.89` to `v0.2.103`
- Update `pico-args` from `v0.4.0` to `v0.4.2`
- Update `rayon` from `v1.5.0` to `v1.5.1`
- Update `serde` from `v1.0.124` to `v1.0.130`
- Update `unicode-segmentation` from `v1.7.1` to `v1.8.0`

## [v2.4.1] 2021-03-18
### Fixed
- Allow empty strings when using the search command line option. For example `--search=""` or `-s=""`

## [v2.4.0] 2021-03-17
### Changed
- Reduce binary size from 3.1M to 2.1M in macOS (aprox)
- Replace `clap` with `pico_args`

## [v2.3.0] 2021-03-13
### Changed
- Update `async-std` from `v1.6.3` to `v1.9.0`
- Update `futures` from `v0.3.5` to `v0.3.13`
- Update `libc` from `v0.2.76` to `v0.2.88`
- Update `log` from `v0.4.11` to `v0.4.14`
- Update `rayon` from `v1.4.0` to `v1.5.0`
- Update `serde` from `v1.0.115` to `v1.0.124`
- Update `termion` from `v1.5.5` to `v1.5.6`
- Update `termios` from `v0.3.2` to `v0.3.3`
- Update `toml` from `v0.5.6` to `v0.5.8`
- Update `unicode-segmentation` from `v1.6.0` to `v1.7.1`
- Update `env_logger` from `v0.7.1` to `v0.8.3`

### Removed
- `smol` dependency

## [v2.2.0] 2020-08-29
### Changed
- Update dependencies
- Update `dirs` to `v3`
- Update `smol` to `0.4`

### Removed
- Debounce behavior in the search engine. All characters will trigger a search.

## [v2.1.0] 2020-05-31
### Changed
- Update `async-std` to `v1.6`.
- Use `smol::block_on` to clean up the screen after the program finishes.

## [v2.0.0] 2020-05-22
### Changed
- Complete rewrite using [`async-std`](https://async.rs/) to build an async architecture.
- The program doesn't wait for the `STDIN` to finish anymore, it can accept an infinte
  stream (although it's not recommended).
- New fuzzy algorithm based on [`fuzzaldrin-plus`](https://github.com/jeancroy/fuzz-aldrin-plus)

### Added
- `--inline` option to display scout UI under the current line in the terminal.
- `--full-screen` option to display scout UI in full screen mode (default).
- Fully customizable UI with a config file. By default in `$HOME/.config/scout.toml`.
- `--config` option to use a custom configuration file path.
- New supported keys: `^e`, `^a` and arrow keys to move around the prompt.
- You can install `scout` using [homebrew](https://brew.sh) with a custom tap
  repository.
- GitHub actions integration.

### Removed
- Travis CI integration.

## [v1.3.0] 2018-01-14
### Changed
- Replaced the green color used to show the matching area in a choice for an
  underline. The color depends on the terminal colorscheme and it could be hard
  to read.

## [v1.2.0] 2017-11-15
### Changed
- Internal: replaced custom made code to handle parallelization with `rayon`
  crate. Now the code is better, faster and nicer.
- Updated dependencies.

## [v1.1.0] 2017-07-19
### Added
- Now if there are more choices than visible lines when you go down
  or up the list of choices, scout scrolls up or down showing choices out
  of the visible lines.

## [v1.0.1] 2017-07-13
### Added
- [trust](https://github.com/japaric/trust/) template to run `scout`
  against different architectures and to generate release packages with
  the binary already compiled for those architectures.

### Fixed
- Fix compilation in `musl` environments.

## [v1.0.0] 2017-07-04
### Added
- Tests for the UI module.
- Custom Error type to control error cases.
- Added docs to all modules.
- Add new option, `--search`, to start `scout` filtering with a query right
  away.

### Changed
- Refactored the UI module into several files.
- Upgraded dependencies.
- New demo in the README.

### Removed
- No more `unwrap()` calls.

## [v0.10.0] - 2017-07-01
### Added
- `rustfmt` config file.
- Instructions of how to use `rustfmt` in the project.
- Add link to scout.vim to README
- Integrate the project with Travis CI.
- Instructions of how to use `clippy` in the project.

### Changed
- Reformatted `rust` source code with `rustfmt`.
- Refactored the code to remove `clippy` offenses.

## [v0.9.2] - 2017-06-29
### Fixed
- Fix the movement of the current choice selected through the choices list

## [v0.9.1] - 2017-06-14
### Fixed
- Add missing changes in Cargo.lock file

## [v0.9.0] - 2017-06-14
### Added
- Info about how to install scout crate.
- Info about tests
- The CHANGELOG.md file (this file).

## Changed
- Do the fuzzy search with different threads in parallel. 

## [v0.8.0] - 2017-05-14
### Added
- You can pipe in a list of items to filter.
- The program will print out the selection.
- You can pipe out the output of the program. It is a good UNIX
- citizen.
- The list of choices to filter adapts to the size of the screen.
- You can move through the list of choices.
- It is UTF-8 aware.

[Unreleased]: https://github.com/jhbabon/scout/compare/v2.6.0...HEAD
[v2.6.0]: https://github.com/jhbabon/scout/compare/v2.5.1...v2.6.0
[v2.5.1]: https://github.com/jhbabon/scout/compare/v2.5.0...v2.5.1
[v2.5.0]: https://github.com/jhbabon/scout/compare/v2.4.1...v2.5.0
[v2.4.1]: https://github.com/jhbabon/scout/compare/v2.4.0...v2.4.1
[v2.4.0]: https://github.com/jhbabon/scout/compare/v2.3.0...v2.4.0
[v2.3.0]: https://github.com/jhbabon/scout/compare/v2.2.0...v2.3.0
[v2.2.0]: https://github.com/jhbabon/scout/compare/v2.1.0...v2.2.0
[v2.1.0]: https://github.com/jhbabon/scout/compare/v2.0.0...v2.1.0
[v2.0.0]: https://github.com/jhbabon/scout/compare/v1.3.0...v2.0.0
[v1.3.0]: https://github.com/jhbabon/scout/compare/v1.2.0...v1.3.0
[v1.2.0]: https://github.com/jhbabon/scout/compare/v1.1.0...v1.2.0
[v1.1.0]: https://github.com/jhbabon/scout/compare/v1.0.1...v1.1.0
[v1.0.1]: https://github.com/jhbabon/scout/compare/v1.0.0...v1.0.1
[v1.0.0]: https://github.com/jhbabon/scout/compare/v0.10.0...v1.0.0
[v0.10.0]: https://github.com/jhbabon/scout/compare/v0.9.2...v0.10.0
[v0.9.2]: https://github.com/jhbabon/scout/compare/v0.9.1...v0.9.2
[v0.9.1]: https://github.com/jhbabon/scout/compare/v0.9.0...v0.9.1
[v0.9.0]: https://github.com/jhbabon/scout/compare/v0.8.0...v0.9.0
[v0.8.0]: https://github.com/jhbabon/scout/tree/v0.8.0
