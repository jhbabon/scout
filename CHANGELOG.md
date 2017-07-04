# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- Tests for the UI module.
- Custom Error type to control error cases.
- Added docs to all modules.

### Changed
- Refactored the UI module into several files.
- Upgraded dependencies.
- New demo in the README.

### Removed
- No more `unwrap()` calls.

## [0.10.0] - 2017-07-01
### Added
- `rustfmt` config file.
- Instructions of how to use `rustfmt` in the project.
- Add link to scout.vim to README
- Integrate the project with Travis CI.
- Instructions of how to use `clippy` in the project.

### Changed
- Reformatted `rust` source code with `rustfmt`.
- Refactored the code to remove `clippy` offenses.

## [0.9.2] - 2017-06-29
### Fixed
- Fix the movement of the current choice selected through the choices list

## [0.9.1] - 2017-06-14
### Fixed
- Add missing changes in Cargo.lock file

## [0.9.0] - 2017-06-14
### Added
- Info about how to install scout crate.
- Info about tests
- The CHANGELOG.md file (this file).

## Changed
- Do the fuzzy search with different threads in parallel. 

## [0.8.0] - 2017-05-14
### Added
- You can pipe in a list of items to filter.
- The program will print out the selection.
- You can pipe out the output of the program. It is a good UNIX
- citizen.
- The list of choices to filter adapts to the size of the screen.
- You can move through the list of choices.
- It is UTF-8 aware.

[Unreleased]: https://github.com/jhbabon/scout/compare/v0.10.0...HEAD
[0.10.0]: https://github.com/jhbabon/scout/compare/v0.9.2...v0.10.0
[0.9.2]: https://github.com/jhbabon/scout/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/jhbabon/scout/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/jhbabon/scout/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/jhbabon/scout/tree/v0.8.0
