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

The work is in progress. The definition files, the builder and the
operators are done. The catalogue is not.
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

## Building

A `SynthDefBuilder` is the `SynthDef` of sclang during a build. It builds
as `SynthDef.new` does, without `optimizeGraph`.

```rust
use scsynth_synthdef::{Arg, NewUGen, Rate, SynthDefBuilder};

// SynthDef(\sine, { |freq = 440| Out.ar(0, SinOsc.ar(freq)) })
let mut builder = SynthDefBuilder::new("sine", vec![Arg::new("freq", 440.0)])?;
let freq = builder.arguments()[0].clone();
let sin = builder.add(NewUGen::new("SinOsc", Rate::Audio).inputs([freq, 0.0.into()]))?;
builder.add(NewUGen::new("Out", Rate::Audio).inputs([0.0.into(), sin]).outputs([]))?;
let def = builder.finish()?;
assert_eq!(def.children.len(), 3);
# Ok::<(), Box<dyn std::error::Error>>(())
```

- `SynthDefBuilder::new` takes the arguments of the graph function, and
  builds their controls at once, as `buildUgenGraph` does. An `Arg` has a
  default value, or an array of them, a rate entry and lags.
  - The rate is the first match of scalar (`ArgRate::Ir` or the prefix
    `i_`), trigger (`Tr` or `t_`) and audio (`Ar` or `a_`). Any other
    argument is at control rate. So the prefix `i_` wins over `Kr`.
  - Each rate has one control UGen, in the order `Control` at scalar rate,
    `TrigControl`, `AudioControl` and `Control` at control rate. If a lag
    is not 0, the control-rate group becomes `LagControl`s of 16 values,
    with the lags as their inputs.
  - The parameter indices follow the order of these groups. The names table
    follows the order of the arguments.
- `arguments` gives one `UGenInput` per argument: an output, or an array of
  outputs for an array.
- `add` adds a UGen at the level of `UGen:new1`: its class, its rate, its
  inputs in the order of the wire, the rate of each output, its special
  index, and whether it is a `WidthFirstUGen`. Every UGen added after a
  width-first UGen runs after it.
- `unary_op` and `binary_op` add a `UnaryOpUGen` and a `BinaryOpUGen` by
  the name of the operator, such as `neg` or `*`. A unary operator runs at
  the rate of its input. A binary operator runs at the highest rate of its
  inputs, in the order demand, audio, control and scalar. A number is at
  scalar rate. The module `operator` gives the special index of each name,
  and the name of each index.
- `variant` adds a variant by control name, from the defaults.
- `finish` collects the constants in the order of creation and of the
  inputs, sorts the UGens as sclang sorts them, and checks the limits.

A `UGenInput` is a number, an `OutputProxy` or an array of them. An
`OutputProxy` gives its rate and the index of its output.

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

## Differences from sclang

- A lag on a scalar, trigger or audio argument: sclang ignores it with a
  warning, and the builder refuses it.
- An empty array as a default: when it is alone at its rate, sclang fails.
  The builder always refuses it.
- The constants: the builder keeps one constant per bit pattern of the f64.
  sclang looks constants up in two hash tables. For 0.0 against -0.0, the
  layout of these tables decides: sclang keeps both, or fails with
  "SimpleNumber-writeInputSpec constant not found: -0.0".
- The variants: the builder writes them in the order of the calls. sclang
  writes them in the order of its dictionary.
- The builder does not run `optimizeGraph`. sclang fuses `*` and `+` into
  `MulAdd`, `+` into `Sum3` and `Sum4`, rewrites `a + b.neg` and
  `a - b.neg`, and removes unused pure UGens.
- An array input: sclang expands it into several UGens, and the builder
  refuses it.
- The operators: sclang folds `* 0`, `* 1`, `* -1`, `+ 0`, `- 0`, `0 -`,
  `/ 1` and `/ -1` before it makes a UGen, and computes the operator of
  numbers. `unary_op` and `binary_op` always make a UGen.

## Tests

`cargo test --all-features` runs every test. It needs no SuperCollider.

The golden files in `tests/fixtures` are definitions that sclang 3.14.1
built and wrote. Each one reads, and writes back to its own bytes.
`generator/fixtures.scd` writes them again:

```sh
sclang -l "$PWD/generator/sclang_conf.yaml" "$PWD/generator/fixtures.scd"
```

- `generator/sclang_conf.yaml` compiles the core class library of
  `/Applications/SuperCollider.app` alone, without the Extensions folder and
  without quarks. For another installation, change its path.
- If `finishBuild` adds, removes or rewires a UGen, the script fails. So no
  golden file holds a graph that `optimizeGraph` rewrites.
- `~/.bin/sclang` changes the directory, so the paths must be absolute.

`generator/catalog.json` holds the tables of the operators. They come from
`Opcodes.h` and `initSpecialSelectors` in the sources of SuperCollider, at
the tag `Version-3.14.1`, and sclang must agree with each special index.
`generator/catalog.scd` writes the JSON again:

```sh
SC_SOURCE=~/Projects/supercollider \
sclang -l "$PWD/generator/sclang_conf.yaml" "$PWD/generator/catalog.scd"
```

`tests/render.rs` renders the JSON into `src/operator/generated.rs`, and
fails if the committed file differs. With `SCSYNTH_SYNTHDEF_BLESS=1` set,
it writes the file instead.

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
