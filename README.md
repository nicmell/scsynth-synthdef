# scsynth-synthdef

The synth definitions of SuperCollider, for Rust. This crate reads and
writes the
[SynthDef file format](https://doc.sccode.org/Reference/Synth-Definition-File-Format.html)
(`SCgf`), the bytes that scsynth loads with `/d_recv`. It will also compile
a graph of UGens the way sclang compiles a `SynthDef`. A catalogue of the
UGens of the core class library will come with it.

```rust
use scsynth_synthdef::{InputSpec, Rate, SynthDef, UGen, read_def_file};

// Out.ar(0, DC.ar(0.5)), the definition "dc".
let def = SynthDef {
    name: "dc".to_owned(),
    constants: vec![0.5, 0.0],
    controls: vec![],
    control_names: vec![],
    children: vec![
        UGen {
            name: "DC".to_owned(),
            rate: Rate::Audio,
            inputs: vec![InputSpec::Constant(0)],
            outputs: vec![Rate::Audio],
            special_index: 0,
        },
        UGen {
            name: "Out".to_owned(),
            rate: Rate::Audio,
            inputs: vec![InputSpec::Constant(1), InputSpec::UGen { index: 0, output: 0 }],
            outputs: vec![],
            special_index: 0,
        },
    ],
    variants: vec![],
};
let bytes = def.as_bytes()?;
let file = read_def_file(&bytes)?;
assert_eq!(file.synth_defs, [def]);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The crate follows SuperCollider 3.14.1. Where sclang and scsynth differ,
the crate refuses what scsynth refuses, and this file lists the difference.

## Status

The work is in progress. The definition files are done. The builder and
the catalogue are not.
[The plan](https://github.com/nicmell/scsynth-synthdef/blob/main/docs/plan.md)
describes version 0.1 and its steps.

## Definition files

A `SynthDef` is a definition after the build: its constants, the initial
values of its parameters, its names table, its UGens in the order in which
scsynth runs them, and its variants. A `DefFile` is the content of a file:
its `Version` and its definitions.

- `read_def_file` reads versions 0, 1 and 2, with any number of
  definitions.
- `write_def_file` writes version 2, as `Collection:writeDef` of sclang
  does. `SynthDef::as_bytes` writes a file of one definition, as
  `SynthDef:asBytes` does.
- `write_def_file_old` writes version 1, as `SynthDefOld` does.

The versions differ in two places:

- Version 2 stores counts and indices as int32, and version 1 as int16.
- Version 0 is version 1 without the variants. scsynth reads the count of
  variants only from version 1 on (`SC_GraphDef.cpp:505`).

The count of definitions and the count of variants are an int16 in every
version.

## Strictness

Reading refuses the bytes that scsynth misreads, and more:

- a truncated file, and bytes after the last definition
- a negative count. scsynth stores it as a uint32 of about four billion.
- a negative index, other than the -1 that marks a constant. scsynth reads
  any negative UGen index as a constant.
- a rate other than 0 to 3. scsynth leaves the rate of the unit unset.
- a name that is not UTF-8. scsynth takes any bytes.
- a magic other than `SCgf`, and a version other than 0, 1 and 2. scsynth
  ignores such a file, and reports no error (`GraphDefLib_Read`).

A `ReadError` gives the offset of the field and what is wrong with it.

### Limits

Writing checks every definition against these limits before it writes a
byte, and reading checks every definition after it reads it. A
`LimitError` names the definition and the limit.

- A definition name takes at most 255 bytes, the most that its length byte
  holds.
- A UGen name, a parameter name and a variant name take at most 31 bytes.
  scsynth throws "name too long (> 31 chars)" at 32 (`ReadName`). The name
  of a variant includes the name of its definition: `<definition>.<variant>`.
- An input reads a UGen before the UGen that reads it. scsynth runs the UGens
  in file order, and its buffer coloring reads the output of a later UGen
  before it is set (`SC_GraphDef.cpp:835-859`).
- An input reads an output that its UGen has, and a constant that the
  definition has. scsynth does not check these indices, and reads out of
  bounds (`DoBufferColoring`).
- A parameter name points at most at the number of parameters. At that
  number, scsynth gives the name 0 channels. sclang writes such a name for
  an empty array beside other controls.
- A variant has one value per parameter.
- Each count fits the integer of the version: `i32::MAX` in version 2, and
  `i16::MAX` in version 1. Each count of definitions and of variants fits
  `i16::MAX`.

## Features

- `serde`: `Serialize` and `Deserialize` for the types of a definition and
  of a file. A `Rate` is its name in lowercase, such as `"audio"`.
  `serde_json` writes a NaN or an infinite float as `null`, and does not
  read it back.

## License

Licensed under either of the Apache License, Version 2.0
(`LICENSE-APACHE`) or the MIT license (`LICENSE-MIT`), at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this crate by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
