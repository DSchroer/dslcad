# Concepts

## Scripts

Everything in DSLCAD starts with a script. A script contains the instructions on 
how to create a model. Scripts can even use other scripts to build complex
projects.

Every script is a `*.ds` file. To create a new script simply create a new file. 

Every script needs to create something. A script that doesn't create something has 
no value in 3D modeling and therefore results in an error.

You can use scripts from other scripts. Simply use a relative path
and execute the script by name. Suppose I have two scripts:

1. `script1.ds`
2. `script2.ds`

If I want to use `script2` from within `script1` I can do the following. Note how I 
use a relative path to identify `script2`.
```
// call script2 from within script1
./script2();
```

## Parts

Simple projects can get away with single 3D models. For more complex projects
it is often useful to break things into multiple parts.

You can make multiple parts in the same script. Just separate each part into its
own statement like so:
```
./part1();

./part2();
```

If you wanted them to be a single part you can join them with a `union` to form 
a single object.

```
./part1() ->left union(right=./part2());
```

## Workflow

DSLCAD is designed with an opinionated workflow in mind. Following this will 
help you build high quality parts quickly. 

For every part that you want to build:

1. Start with 2D sections of it. Currently, the `face` function is the simplest
way to create any 2D sketch. 

2. Extrude your face into a 3D section using `extrude` or `revolve` to bring your
face into the 3D world. 

3. Combine 3D sections using `union`, `difference` and `intersect` to cut out the 
final dimensions of your part. 

4. [OPTIONAL] Use `chamfer` or `fillet` to add a polished look to the part. 

