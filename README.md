# ModelScript

ModelScript is a programming language & interpreter for building 3D models. 

**WARNING!!!** ModelScript is in very early stages. Documentation and usability is a work 
in progress. There will be breaking changes. 

## Usage

```
cargo run -- ./examples/table.ex -o table.stl

#OR

model-script ./examples/table.ex -o table.stl
```

## Development Status

- 3D Editing:
  - Primitives
    - [X] Cube
    - [X] Cylinder
  - Transforms
    - [X] Translation
    - [X] Rotation
    - [X] Scaling
    - [ ] Mirror
  - Solid Modeling
    - [X] Union
    - [X] Difference
    - [ ] Intersect
  - Tools
    - [X] Chamfer
      - [ ] Mask
    - [X] Fillet
      - [ ] Mask
- Language:
  - Literals:
    - [X] Numbers
    - [X] Booleans
    - [X] Text
  - [X] Immutable Variables
  - [X] Function Calls
  - [X] Cross Document Calls
  - [X] Pipe Operator
- 2D Design:
  - [X] Points
  - [X] Lines
  - [X] Arcs
  - [X] Extrude
  - [X] Rotational Extrude
- CLI interface:
  - [ ] Variable overriding
  - [ ] Unified output format
  - [X] 3D output display
