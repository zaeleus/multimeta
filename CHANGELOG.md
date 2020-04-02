# Changelog

## Unreleased

### Added

  * editor: Add album and song id to editor. Use `i` at the edit prompt to edit
    an album or song id. This is useful when a name cannot be parameterized,
    e.g., one with just symbols (":(", "[#]", etc.), or a name is a duplicate
    within an artist or album.

  * melon: Parse "베스트" ("best") album kind as an LP. This is typically a
    compilation album.

## [0.3.0] - 2019-12-20

### Added

  * melon: Replace prime (′) with quotation mark.

### Fixed

  * Names with double quotes would result in invalid TOML. Double quotes in
    names are now escaped. For example, `The year of "YES"` is now rendered as
    `The year of \"YES\"`.

## [0.2.0] - 2019-10-01

### Added

  * melon: Parse "옴니버스" ("omnibus") album kind as an LP. Omnibus albums are
    likely to be either an EP or LP, but since it's typically a collection of
    many songs, it's assumed to be an LP.

### Changed

  * The line editor changed from [Readline] to [Rustyline]. Rustyline provides
    the same functionality that is used in the album editor without requiring
    an out-of-project dependency.

  * mora: Requests are now made over https. Previously, mora only supported
    http.

[Readline]: https://tiswww.case.edu/php/chet/readline/rltop.html
[rustyline]: https://github.com/kkawakam/rustyline

### Fixed

  * Fixed a panic when collecting artist IDs and checking for existence.
    This failed when the destination prefix was the current directory (i.e.,
    `--output .`) and the artist ID was a single character (e.g., `i.toml`).

## 0.1.0 - 2019-09-04

  * Initial release

[0.2.0]: https://github.com/zaeleus/multimeta/compare/v0.1.0...v0.2.0
