# Proposal: Views & Architectural Drawings

Status: Draft
Related: TODO "Visualization features", `--screenshot`

## Summary

DSLCAD can build and render 3D parts, and a single view can be captured with
`--screenshot`. What it cannot do is *author* a view or a 2D drawing: camera
angles only exist on the command line, and there is no way to produce
dimensions or labels that survive into the preview and the exported image.

This proposal adds a small drawing layer to the language. A view is the single
combined concept: it holds the camera *and* the drawing content, so there is no
separate "drawing" object.

`view` is a top-level **statement**, not an operator. It is never piped into and
never stored in a variable, so it gets its own grammar instead of a function
call. Everything a view draws stays an ordinary expression.

- `view "Name" (...) { ... }` is a new statement that groups a camera with the
  layers it draws.
- `measure(...)` and `dimension(...)` express distance measurements.
- `label(...)`, `leader(...)`, `level(...)`, `north(...)`, `centerline(...)`
  and `title(...)` cover the usual drawing annotations.

Annotations remain ordinary values, so they pipe and nest naturally
(`part -> dimension(...)`) and can be built with `func`. Only the view wrapper
is syntactic.

## Motivation

A parametric CAD language is well suited to drawings, because dimensions are
already parameterized. Today that information has to be re-entered by hand after
rendering. Keeping the drawing in the `.ds` file means:

- a model change updates its dimensions automatically;
- the same drawing definition drives the live preview and the screenshot;
- drawings become versionable, reviewable source instead of editing pixels.

It also completes the workflow in `concepts.md`, which already encourages "2D
first, then 3D": drawings are the natural 2D output of a 3D part.

## Design principles

1. **One new statement, no new value.** `view` is a statement because it is
   never piped or assigned, so it is not a first-class value. Annotations stay
   expressions.
2. **Reuse existing semantics.** `view.angle` reuses `AxisAngles`; dimension
   lines reuse the line renderer; labels reuse the font importer.
3. **Annotations never affect framing.** They are presentation, so they are
   excluded from `BoundingBox::from_parts` and camera auto-focus.
4. **Minimal grammar.** One reserved word and one statement form. Everything
   else is ordinary functions and values.

## Values and statements

New runtime value, alongside `Shape` / `Plane` / `Line` in
`crates/dslcad/src/runtime/value.rs`:

| Kind | Produced by | Meaning |
| --- | --- | --- |
| `annotation` | `dimension`, `label`, `level`, `north`, `centerline`, `title` | drawable overlay |

`annotation` is one enum value with a variant per kind, which keeps the
signature machinery and the protocol simple. It is what `view` blocks collect
and what the viewer overlays.

The view itself is not a value; it is a new `Statement::View`.

## 1. The `view` statement

```dslcad
view "Front elevation" (angle="front", projection="orthographic", zoom=1.1) {
    bracket;
    dimension(start=point(0, 0, 0), end=point(60, 0, 0), offset=-8);
    dimension(start=point(0, 0, 0), end=point(0, 0, 20), offset=-8);
    label(text="Bracket", at=point(30, 40, 0), size=5);
}
```

Grammar:

```
view [ "name" ] [ ( camera arguments ) ] { layer statements }
```

- The **name** is an optional string. When omitted the view is unnamed and the
  preview/CLI fall back to the view's index.
- The **camera arguments** reuse the normal call-argument syntax and are named.
- The **block** is the view's layers: ordinary statements, exactly like a scope.
  Geometry contributes to the shared model scene, while annotations belong to
  the view.
- `view` is only valid at the **top level** of a document. It cannot be piped,
  assigned, or nested inside a scope or `func`.

Camera arguments:

- `angle=[text]` — a friendly name (`"front"`, `"top"`, `"right"`, `"left"`,
  `"back"`, `"bottom"`, `"iso"`) or the existing CLI form `"x0y0z0"`. Friendly
  names are mapped to `AxisAngles` constants.
- `projection=[text]` — `"perspective"` (default) or `"orthographic"`.
- `zoom=[number]` — magnification of the fit-to-view distance (default 1).
- `target=[point]` — explicit look-at point; when unset the model centre is used.
- `fit=[bool]` — fit the camera to the model bounds (default `true`). Ignored
  when `target` is set.
- `show=[list]` — layer-visibility defaults: any of `"model"`, `"points"`,
  `"lines"`, `"mesh"`, `"annotations"`.

The simplest view has no name, no arguments and just draws geometry:

```dslcad
view {
    cube();
}
```

This generalizes the current `--screenshot <angle> <zoom>` arguments in
`crates/dslcad/src/main.rs`. The CLI form keeps working; it is equivalent to a
single implicit view.

## 2. Distance measurements

The measurement query and its annotation are separate, so a value can be used
in arithmetic before it is drawn:

```dslcad
measure(start=point, end=point)   // -> number, distance in mm
measure(shape=shape)              // -> point, the x, y and z extents
```

```dslcad
dimension(start=point, end=point, offset=[number], axis=[text],
          text=[text], units=[text], precision=[number], arrow=[text])
dimension(radius=number, center=point, at=point, units=[text])                // radial
dimension(angle=number, center=point, start=point, end=point, units=[text])  // angular
```

- `offset=[number]` — signed distance from the measured line to the dimension
  line (extension lines are generated automatically).
- `axis=[text]` — constrain the measurement/extension direction to `"x"`,
  `"y"` or `"z"` (`"aligned"` by default).
- `text=[text]` — override the label. When unset, the label is the measured
  value formatted with `precision` (default 1) and the chosen unit.
- `units=[text]` — display unit for the auto label: `"mm"` (default), `"cm"`,
  `"m"`, `"in"` or `"ft"`. Units are per annotation, so one drawing can mix
  them.
- `arrow=[text]` — `"closed"` (default), `"open"`, `"tick"` or `"dot"`.

`measure(...)` always returns millimetres, so the number stays canonical and
units only affect presentation. To print a converted value yourself, use the
`convert(value=number, from=[text], to=[text])` helper with `format`:

```dslcad
dimension(start=point(0, 0, 0), end=point(60, 0, 0), offset=-8, units="in");
label(text=format("{v}\"", v=round(convert(60, "mm", "in"), 3)), at=point(30, 40, 0));
```

Because dimensions are ordinary values, dimension chains are just `map`:
project several offsets over `range(...)`.

## 3. Labels and components

```dslcad
label(text=text, at=point, anchor=[point], size=[number],
      plane=[text], billboard=[bool])
leader(text=text, at=point, to=point)       // arrow plus note
level(z=number, text=[text], at=[point])    // elevation datum triangle
north(angle=[number], at=[point])           // plan north arrow
centerline(start=point, end=point)          // dash-dot axis line
title(text=text, subtitle=[text], scale=[text])
```

- `anchor=[point]` — point on the leader; defaults to `at`.
- `size=[number]` — cap height in mm (default 4).
- `plane=[text]` — text plane `"xy"`, `"yz"` or `"xz"` (default `"xz"`, facing
  the viewer).
- `billboard=[bool]` — keep text facing the camera (default `true`).

Labels are rendered as world-space geometry in phase 1 (the font importer in
`crates/dslcad/src/resources/ttf_loader.rs` already turns text into a plane),
then as a viewer text overlay in phase 2 for crisp scaling.

## End-to-end example

```dslcad
var bracket = cube(x=60, y=40, z=20)
    -> difference(cylinder(radius=6, height=40) -> translate(x=10, y=20));

view "Front elevation" (angle="front", projection="orthographic", zoom=1.1) {
    bracket;
    dimension(start=point(0, 0, 0), end=point(60, 0, 0), offset=-8);
    dimension(start=point(0, 0, 0), end=point(0, 0, 20), offset=-8);
    label(text="Bracket", at=point(30, 40, 0), size=5);
}
```

A file can emit several views; each is a separate render target, so the preview
lists them and `--views` writes one png per view:

```dslcad
view "Front" (angle="front", projection="orthographic") {
    bracket;
    dimension(start=point(0, 0, 0), end=point(60, 0, 0), offset=-8);
}

view "Top" (angle="top", projection="orthographic") {
    bracket;
    dimension(start=point(60, 0, 0), end=point(60, 40, 0), offset=-8);
}
```

## CLI and preview integration

### CLI

- `--screenshot` accepts an optional view name in addition to angle/zoom:
  `dslcad ./part.ds --screenshot --view "Front elevation"` renders that view's
  camera and annotations.
- `--views front,top,right` renders one png per named view, named
  `<stem>.<view>.png`.
- `--screenshot` with no arguments keeps the current behavior and uses the
  file's first view if one exists, otherwise the model's default framing.

### Preview

- The `View` menu gains a **Views** submenu listing every view in the file;
  selecting one animates the camera to it and shows that view's annotations.
- A **Layers** submenu toggles `Dimensions`, `Labels` and `Annotations`, in the
  same style as the existing grid/mesh toggles in
  `crates/dslcad_viewer/src/editor/gui/view_menu.rs`.
- `Orthographic` becomes a checkable menu item (`CameraCommand::UseOrthographic`
  already exists).

## Rendering pipeline changes

### Parser

- **Lexer**: add `#[token("view")] Token::View`. `view` becomes a reserved word.
- **AST** (`crates/dslcad/src/parser/syntax_tree.rs`): add a statement variant

  ```rust
  pub enum Statement {
      Variable(Variable, Span),
      CreatePart(Expression, Span),
      View(View, Span), // new
  }

  pub struct View {
      pub name: Option<String>,
      pub arguments: VecDeque<Argument>,
      pub body: Vec<Statement>,
  }
  ```

- **Grammar** (`crates/dslcad/src/parser.rs`): `parse_statement` dispatches
  `Token::View` to a new `parse_view_statement`, which reads the optional string
  name, the optional `(...)` arguments via the existing `parse_call_arguments`,
  and the `{ ... }` body via `parse_scope`. The parser rejects a `view` that is
  not at the top level of a document.
- **Visitor** (`crates/dslcad/src/parser/syntax_visitor.rs`): extend
  `StatementVisitor` with `visit_view`.

### Protocol

In `crates/dslcad_storage/src/protocol.rs`, extend `Render`:

```rust
pub struct Render {
    pub parts: Vec<Part>,
    pub stdout: String,
    pub views: Vec<ViewDef>, // new
}

pub struct ViewDef {          // new
    pub name: String,
    pub angle: AxisAngles,
    pub projection: Projection,
    pub zoom: Option<f32>,
    pub target: Option<Point>,
    pub show: ShowFlags,
    pub annotations: Vec<Annotation>,
}
```

- `Annotation` is a serde enum: `Polyline`, `Arrow`, `Text`.
- A `ViewDef` carries the camera and annotations, so `--screenshot` and the
  preview can select a view without re-parsing the file. Each view references
  the shared `Render::parts` scene.
- Geometry encountered outside any view (plain top-level values) becomes an
  implicit, unnamed view, which preserves today's behavior.
- `BoundingBox::from_parts` ignores `annotations` so auto-focus and screenshot
  framing are unaffected.

### Viewer

- New `crates/dslcad_viewer/src/editor/annotation.rs`:
  - arrows and dimension lines reuse the batched `LineMaterial` from
    `editor/lines.rs`;
  - text uses a font atlas (phase 2) or pre-baked geometry (phase 1).
- `RenderState` gains `show_annotations`, `show_dimensions`, `show_labels`, and
  tracks the active view.
- `CameraCommand::Focus` accepts an explicit view target for named views.

### Runtime

- The `Engine`'s `StatementVisitor` gains `visit_view`: it evaluates the camera
  arguments, evaluates each body statement, splits the results into scene
  geometry and annotations, and builds a `ViewDef`.
- `view` is **not** registered in the library. Annotations (`dimension`,
  `label`, `level`, `north`, `centerline`, `title`) are registered in
  `crates/dslcad/src/library/drawing.rs` under `Category::Drawing`.
- `Value` gains the `Annotation` variant; `flatten`/`to_output` treat it as a
  non-geometric value.
- `RuntimeError` gains variants for invalid views, surfaced by the existing
  `error_printer`.

## Phasing

1. **Statement + annotations, no protocol change.**
   Parse `view`; `measure`, `dimension`, `label` as functions; labels emit text
   geometry; views are flattened to parts plus a camera. Preview and screenshot
   work.
2. **First-class views.**
   `Render.views`/`ViewDef`/`Annotation`; viewer text and arrow rendering; layer
   toggles; `--view` and `--views`.
3. **Components and multi-view output.**
   `title`, `level`, `north`, `centerline`; multi-view screenshots;
   named-view menu.

## Compatibility

- `view` becomes a reserved word. A model that used `view` as a variable or
  function name must rename it; everything else is additive.
- `--screenshot x90y45 2` remains valid and keeps its current meaning.
- The bincode `Render` struct is extended; the viewer and CLI are versioned and
  built together, so this is an internal protocol change.

## Open questions

- Should views be allowed inside `func` and scopes so a reusable view template
  can be shared? Today they are top level only, and `func` cannot return one.
- Per-view geometry: views currently share one model scene. Should a view be
  able to show a subset of the geometry, for example an assembly detail view?
- Should annotations participate in exports (`3mf`/`step`) at all, or remain
  preview/screenshot only? Recommendation: preview/screenshot only.
