# Changelog

## Unreleased

### Changed

  * The line editor changed from [Readline] to [Rustyline]. Rustyline provides
    the same functionality that is used in the album editor without requiring
    an out-of-project dependency.

[Readline]: https://tiswww.case.edu/php/chet/readline/rltop.html
[rustyline]: https://github.com/kkawakam/rustyline

### Fixed

  * Fixed a panic when collecting artist IDs and checking for existence.
    This failed when the destination prefix was the current directory (i.e.,
    `--output .`) and the artist ID was a single character (e.g., `i.toml`).

## 0.1.0 - 2019-09-04

  * Initial release
