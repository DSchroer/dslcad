# Proposal: View composition

Status: Implemented
Related: `drawings.md` (supersedes its "Per-view geometry" open question),
TODO "Visualization features", `--screenshot`, resource imports.

## Implementation notes

The feature is implemented on `master` with two deliberate simplifications
from the sketches below:

- **`model()` embeds the default model** rather than tracking an
  `include_model` reference. `model()` returns the top level values as an
  ordinary list, and a view's `parts` are whatever its body produced. This is a
  little more meshing work per view but is simpler and, unlike a reference,
  correct for views exported by imported documents (whose default model is not
  the importer's shared scene).
- **`model` is not a lexer token.** It stays a contextual builtin: the parser
  allows the identifier, and the runtime resolves `model()` to the default
  model whenever a view is being evaluated (through `func` calls too). A
  `var model` inside a view is a runtime error; outside a view the name is
  ordinary.
- Imported views propagate into `Render.views` and are reachable as
  `part.Front()` through the shared namespace, but their names are not yet
  qualified (`part.Front`). See Open questions.

## Summary

Views today are a camera plus implicit access to one shared model scene. That
made the shared scene unavoidable: a view could not be empty, could not carry
its own geometry, and could not be reused as a unit. This proposal makes views
pure compositions of values:

1. **Empty by default.** A view draws exactly its body. `view Top {}` draws
   nothing.
2. **`model()`** is a builtin value for the document's default model (geometry
   and its annotations). It is **reserved only inside a view body**; outside a
   view, `model` is an ordinary identifier.
3. **Views are callable.** `view Top { base(); }` includes another view's
   content.
4. **Documents export their views.** Importing a document gives access to the
   views it declared.

The original problem -- modifying the model for one view without disturbing the
default view -- falls out of 1 and 2:

```dslcad
cube(x=60, y=40, z=20)
    -> difference(cylinder(radius=6, height=40) -> translate(x=30, y=20));

view Front(angle="front", projection="orthographic") {
    model();
    dimension(start=point(x=0, y=0, z=0), end=point(x=60, y=0, z=0), offset=-8, plane="xy");
}

view SectionAA(angle="right", projection="orthographic") {
    model() -> intersect(cube(x=240, y=160, z=80) -> center());
    dimension(start=point(x=0, y=0, z=0), end=point(x=60, y=0, z=0), offset=-8, plane="zx");
}
```

## Semantics

### Default model

- The **default model** `D` is the set of values produced by the document's
  top-level statements that are **not** views.
- `D` is what exports (`3mf`/`step`), what the free camera frames, and what
  `model()` returns.
- Top-level annotations remain global overlays (as today).

### Empty views

- A view renders `include_model ? D : {}` plus its own body values.
- Body values split into view-local parts and view-local annotations.
- View-body geometry is **never** merged into `D` (replaces the current
  behavior in `crates/dslcad/src/lib.rs:106-120`).

### `model()`

- Resolves to `D` as a **reference**, so calling it twice draws once and it
  composes in pipes: `model() -> intersect(...)`, `model() -> slice(...)`.
- `model()` includes `D`'s parts and its global annotations.
- `model` is reserved **only inside a view body**: there it always means `D`
  and cannot be shadowed by a `var`. Outside views it is a normal identifier and
  may be used as a variable or function name.
- The builtin is active for **anything evaluated while a view is active**,
  including a `func` defined outside the view and called from it. Resolution is
  contextual (engine view depth), not lexical.
- Calling `model` at the top level refers to whatever `model` is bound to
  there; if nothing is bound, it is an ordinary undefined-name error, not the
  builtin.

### Calling views

- A view name used as an expression denotes a call that returns that view's
  **content**: its body's parts and annotations.
- The camera is **not** composed. A nested view never changes the caller's
  camera, `projection`, `zoom`, `target`, `fit`, or `show`. This is the
  "cameras do not inherit" rule.
- Views are evaluated in document order after `D`; a view may call only views
  declared before it (definition-before-use). Because content is computed
  eagerly and calls return the computed content, call cycles are impossible.
- A call contributes content to the *caller's* view-local scene, not to `D`.

```dslcad
view base() { model(); }

view top() {
    base();                 // base's content; top's camera
    dimension(...);
}
```

### Exported views

- A document's `ScriptInstance` carries the views it declared, alongside its
  parts and variables.
- A view name lives in the **same namespace as variables** (see below), so an
  importer reaches it by ordinary property access: `./part.ds().Front()`. No
  separate view namespace is introduced.
- When an imported document is **emitted at the top level**
  (`./part.ds();`), its views are added to `Render.views` with a qualified name
  (`part.Front`), so the preview and `--views`/`--screenshot` can select them.
- Imported views keep their own cameras; each is selectable independently.
  Their framing assumes their model is present, which holds when the import is
  emitted.
- A views-only document is importable (no `NoReturnValue` error).

### One namespace

- Views and variables share a single namespace per document: declaring
  `view front` where `var front` already exists is an error, and vice versa.
- This makes view names addressable exactly like variables, so imported views
  need no special spelling (`part.Front()`), and there is no ambiguity to
  resolve.
- `view` statements are **top level only**. They cannot appear inside a scope,
  `func`, or another view body. Calls to views may appear wherever an
  expression may, but a `view` declaration may not.

## Evaluation model

The root document (and each imported document) is evaluated in two phases:

1. **Default pass.** Evaluate top-level non-view statements in order. Record
   the produced values as `D` and keep mutating the scope as today.
2. **View pass.** In document order, evaluate each `view`:
   - resolve camera arguments,
   - evaluate the body with `model()` bound to `D`,
   - split body values into view-local parts/annotations,
   - register the view name in scope as a constant returning its content,
   - append a `ViewDef` to the document's view list.

This makes `model()` position-independent (a view declared above some geometry
still sees all of `D`) and makes view-to-view calls deterministic.

Nested documents imported during phase 1 run the same two-phase evaluation and
return their own `D` and views.

## Data model

### `crates/dslcad/src/runtime/value.rs`

- `Value::Model` -- marker returned when resolving `model`; invocation returns
  the engine's `D`.
- `Function::Constant { value: Value }` -- a callable that simply returns a
  stored value, used for view names and imported views.
- `ViewValue` gains:
  - `parts: Vec<Value>` (view-local geometry, kept separate from `layers`)
  - `include_model: bool`
- `flatten`/`to_output`: `Value::Model` flattens to `D` (or is resolved before
  flatten); `Function::Constant` produces no output.

### `crates/dslcad/src/runtime/script_instance.rs`

- Add `views: IndexMap<String, ViewValue>` for rendering/qualification.
- Register each view name in the instance's variable namespace as a
  `Function::Constant` so `part.Front()` resolves through `Access::get` like any
  other variable.
- `new` must allow a document with no parts when it declares views.

### `crates/dslcad_storage/src/protocol.rs`

- `ViewDef` gains `parts: Vec<Part>` and `include_model: bool`.
- `Render.parts` stays the default model `D`.

## Parser and lexer

- `crates/dslcad/src/parser/lexer.rs`: **no new token**; `model` is not a global
  keyword. It is a contextual builtin, so it parses as an ordinary `Reference`
  and is resolved at evaluation time.
- `crates/dslcad/src/runtime.rs`: while evaluating a view body (and anything
  reachable from it), `model` resolves to `Value::Model`;
  `visit_invocation` (`runtime.rs:540` onward) returns `D` when the callee is
  `Value::Model`.
- No grammar change for view calls: `base()` already parses as an invocation,
  and `./part.ds().Front()` already parses as resource + property + invocation.
- Enforce the single namespace at declaration: a `view` whose name is already
  bound (variable, view, or parameter) is an error, and a `var` whose name
  matches an existing view is likewise an error.

## Runtime

- `Engine` tracks a **view depth** (a counter set while evaluating a view body,
  including nested `func` calls made from it). `model` resolution is driven by
  this flag, not by a lexical scope binding, so it reaches through calls.
- `crates/dslcad/src/runtime.rs:277` (`visit_view`) is reworked to the two-phase
  behavior: build `ViewDef { include_model, parts, annotations, camera }`,
  register the callable constant, and return `Value::View`. Registration fails
  if the view name already exists in the namespace.
- `visit_reference` (`runtime.rs:525`) resolves the name `model` to
  `Value::Model` whenever the view depth is greater than zero, regardless of the
  lexical scope. `visit_variable` (`runtime.rs:232`) rejects a `var model`
  declared while the view depth is greater than zero, and accepts it otherwise.
- `visit_variable` also fails if the declared name is already a view, enforcing
  the single namespace.
- `crates/dslcad/src/runtime.rs:540` (`visit_invocation`): when the callee
  resolves to `Value::Model`, return `default_model`; when it resolves to
  `Function::Constant`, return the stored value.
- Root-document evaluation is split so views run after `D` is complete. The
  existing `eval_statements`/`eval_block` (`runtime.rs:104-143`) stay for
  functions and scopes.
- `CallPath::Document` attaches the resource name to the `ScriptInstance` so
  imported views can be qualified.

## Render pipeline

`crates/dslcad/src/lib.rs:90-151` (`render`):

- Collect `D` from non-view top-level values and views from top-level
  `Value::View`s plus emitted imported documents.
- Each local view becomes `ViewDef` with `include_model`, `parts`,
  `annotations`, and its camera.
- Imported views are added with qualified names.
- `Render.parts` = output of `D`; imported view geometry stays view-local.

## Viewer

`crates/dslcad_viewer/src/editor/rendering.rs`:

- `RenderState` selects the scene per active view: `D` when no view is active
  or the view has `include_model`, plus the view's `parts`.
- Mesh/edge/point/line renderers (`rendering.rs:138`, `:190`, `:229`, `:274`)
  iterate the selected scene instead of always `model.parts`.
- `aabb` (`rendering.rs:73`) frames the selected scene, so detached views focus
  on their own geometry.
- Annotations (`rendering.rs:103`) stay global + active view.
- `views_panel.rs` lists qualified names for imported views.

## CLI

`crates/dslcad/src/main.rs:345-377`:

- `--view` accepts qualified names (`part.Front`) in addition to local ones.
- `--screenshot` with no name keeps using the first view.
- `--views` renders one PNG per view, including imported ones.

## Compatibility

Breaking (pre-1.0):

- Empty views draw nothing; annotation-only views must add `model();`. Affected:
  `examples/dimensions.ds` and the `it_collects_views` /
  `it_allows_views_without_values` tests (`crates/dslcad/src/lib.rs:464-498`).
- View-body geometry no longer contributes to `Render.parts`; models that relied
  on a view to define the part must emit it at the top level.
- `model` is reserved only inside a view body: a `var model` declared inside a
  view is now an error. Top-level and `func` use of the name is unaffected.

Migration is mechanical: add `model();` to any view that should show the
default model, and move geometry that used to live in a view to the top level.

## Phasing

1. **Core.** `model()`, empty views, view-local parts, two-phase evaluation,
   protocol/render/viewer split. Update tests and examples.
2. **View calls.** Register view names as constants; definition-before-use; the
   "cameras do not inherit" rule.
3. **Document export.** `ScriptInstance.views`, view names in the shared
   namespace, qualified names, emitted imports contribute views, CLI/UI
   addressing.

## Tests

- `view Top {}` renders no parts; free camera frames `D`.
- `model()` in a view renders `D`; calling it twice does not duplicate.
- A `func` defined at the top level that calls `model()`, invoked from a view,
  renders `D`; invoked at the top level, it does not resolve the builtin.
- `model() -> intersect(...)` yields a section without altering `D` or exports.
- `view base { model(); } view top { base(); }` composes content, ignores
  `base`'s camera.
- Forward/cyclic view references produce a clear error.
- `./part.ds()` exposes `part.Front()`; emitting it adds `part.Front` to
  `Render.views`.
- `view front` after `var front` (and vice versa) is an error.
- Views-only document imports without error.

## Resolved decisions

- `model()` includes the default model's geometry **and** its annotations.
- `model` is reserved only inside a view body; outside views it is an ordinary
  identifier. The binding reaches through calls: a `func` defined outside a view
  resolves `model` when invoked from within a view, because resolution is driven
  by the engine's view depth rather than lexical scope.
- Views and variables share one namespace: `view front` and `var front` cannot
  coexist.
- `view` statements are top level only.
- Imported views are addressed through the shared namespace
  (`part.Front()`); no separate `.views` object.

## Open questions

- Qualified-name collisions in `Render.views` when two emitted documents each
  export a view with the same name (`part.Front` vs `part2.Front`). Derive the
  qualifier from the resource path; decide the disambiguation rule.
- Should emitting an import auto-register its views, or should that require an
  explicit form? Current recommendation: auto-register on a top-level emit only.
