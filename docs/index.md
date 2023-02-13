# Getting Started

DSLCAD is a programming language & interpreter for building 3D models.

Inspired by [OpenSCAD](http://openscad.org/), it has a language and 3D viewer to simplify the modeling experience.

![screenshot](./screenshot.png)

## Installation

Download the latest DSLCAD from the [Releases](https://github.com/DSchroer/model-script/releases) tab of this repo.

You can find pre-built binaries for:

- Windows
- MacOS
- Linux

Download the zip file for your system and extract it.

To start the DSLCAD viewer simply run the program.

### System Requirements

The core compiler and CLI should run on just about any system.

In order to use the GUI you need to meet the [bevy system requirements](https://github.com/bevyengine/bevy/blob/latest/docs/linux_dependencies.md).
Notably you should have a vulkan enabled graphics driver.

## Usage

For basic editing, run DSLCAD and use the GUI.

For automated usage, list the CLI options with `dslcad --help`.

To see what can be built check out the [examples](https://github.com/DSchroer/dslcad/tree/master/examples) folder.

## Hello World

To create a DSLCAD program. Simply create a new `*.ds` file or use `File > New`.
Edit the file and add the following code:

```
// create a simple cube
cube();
```

Now run the program and open the file by following `File > Open` and selecting
the file you just created.

If everything worked you should see a cube in your editor like this:
![hello](./hello.png)