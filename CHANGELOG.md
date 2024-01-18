# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Camera auto-focus in preview mode
- Support for nested scopes using `{}` syntax

## [v0.0.3]

### Added
- `slice` operator to take cross-sections of 3D parts
- `offset` operator to modify a 2D part
- Ability to skip the parameter name in `->` operations
- Ability to skip the parameter name by parameter order
- `--log` parameter to set the log level
- `--preview` parameter to view the preview window
- WASM build for browser support
- Import from `stl` file
- Import from `ini` file
- Export to `3mf` format
- Export to `raw` format

### Changed
- Removed editor by default

### Removed
- Export to `txt` file
- Export to `stl` file

## [v0.0.2] - 2023-03-10

### Added
- New `center()` function for 2D & 3D primitives
- Export to `txt` file support
- Trig functions for `sin`, `cos`, `tan`, `sqrt`
- Text operations `format` and `formatln`
- Math `ceil` and `floor`
- Escape sequences for `\r`, `\n`, `\"`, `\\`, `\t`

### Changed
- Removed the `center` parameter from 2D primitives. Instead of 
`square(center=true)` use `square() ->shape center()`.

### Fixed
- Use system UI scale factor
- Use Create, Modify and Remove events for file watcher
- Order of operations
- Windows builds crashing randomly
- Crash on opencascade operations with invalid arguments
- Files not static linking on win32
- Docs update CI process

## [v0.0.1] - 2023-02-10

First full release!

