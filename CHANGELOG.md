# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Import from `svg` file
- `--screenshot` parameter to render a single view to a png file with optional axis angles (`x90y45`)
- `bend` operator to bend a 3D shape around an axis
- `normalize` operator to uniformly scale a line, plane or shape so the largest side of its bounding box is 1 unit long
- `simplify` operator to reduce detail: it decimates a line or plane within a tolerance, and merges same-domain faces and edges of a shape

### Modified
- Semicolons are now optional: statements can end at a newline, like JavaScript automatic semicolon insertion
- Render all of the lines of a part in a single draw call
- Only report unique edges and vertices for shapes
- Preview uses a CAD style viewport with a ground grid, color coded axes (x red, y green, z blue) and a corner axis gizmo
- Lines and points use a bright wireframe color when drawn on the viewport background, such as for 2D parts or hidden meshes
- `Grid` toggle in the `View` menu

### Fixed
- Slow preview and screenshot rendering for parts with many edges
- Slow `bend` on shapes with many faces by projecting pcurves locally and repairing them with `SameParameter` instead of a full `ShapeFix`

## [v0.0.5]

### Added
- Added the `error` function to indicate invalid states

### Modified
- Improved printing of errors

## [v0.0.4]

### Added
- Camera auto-focus in preview mode
- Support for nested scopes using `{}` syntax
- Support for nested documents using `func {}` syntax
- CLI argument support via the `--argument` parameter

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

