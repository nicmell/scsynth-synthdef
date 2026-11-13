# scsynth-synthdef

The synth definitions of SuperCollider, for Rust. This crate compiles a
graph of UGens the way sclang compiles a `SynthDef`, into the
[SynthDef file format](https://doc.sccode.org/Reference/Synth-Definition-File-Format.html)
(`SCgf`). scsynth loads these bytes with `/d_recv`. The crate reads them
back, strictly. It carries a catalogue of the UGens of the core class
library.

```rust
use scsynth_synthdef::{Arg, SynthDefBuilder, read_def_file};

// SynthDef(\sine, { |freq = 440, amp = 0.1| Out.ar(0, SinOsc.ar(freq) * amp) })
let mut builder = SynthDefBuilder::new("sine", vec![Arg::new("freq", 440.0), Arg::new("amp", 0.1)])?;
let (freq, amp) = (builder.arguments()[0].clone(), builder.arguments()[1].clone());
let sine = builder.ugen("SinOsc", "ar", &[("freq", freq.into())])?;
let sound = builder.binary_op("*", sine, amp)?;
builder.ugen("Out", "ar", &[("bus", 0.0.into()), ("channelsArray", sound.into())])?;
let def = builder.finish()?;

// The bytes for /d_recv, and back.
let bytes = def.as_bytes()?;
assert_eq!(read_def_file(&bytes)?.synth_defs, [def]);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The crate follows SuperCollider 3.14.1. For the graphs that it builds, it
writes the bytes that sclang writes, except for the differences that this
file lists. Where sclang and scsynth differ, the crate refuses what scsynth
refuses. It does no I/O, and it builds for `wasm32-unknown-unknown`.

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

// SynthDef(\sine, { |freq = 440| Out.ar(0, SinOsc.ar(freq)) }), at the level of new1
let mut builder = SynthDefBuilder::new("sine", vec![Arg::new("freq", 440.0)])?;
let freq = builder.arguments()[0].clone();
let sine = builder.add(NewUGen::new("SinOsc", Rate::Audio).inputs([freq, 0.0.into()]))?;
builder.add(NewUGen::new("Out", Rate::Audio).inputs([0.0.into(), sine]).outputs([]))?;
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
- `ugen` adds a UGen by its class name and the selector of a rate method,
  `ar`, `kr`, `ir` or `new`, with the arguments by name. The catalogue
  gives the order of the arguments, their defaults, the wiring and the
  rule of `checkInputs`. An `ArgValue` is an input, or the text of a symbol
  argument such as the command name of `SendReply`. A single input goes
  where the method takes an array, as `asArray` makes it. An array
  expands the UGen, and `mul` and `add` go through `madd`.
- `add` adds a UGen at the level of `UGen:new1`: its class, its rate, its
  inputs in the order of the wire, the rate of each output, its special
  index, and whether it is a `WidthFirstUGen`. Every UGen added after a
  width-first UGen runs after it.
- `unary_op` and `binary_op` add a `UnaryOpUGen` and a `BinaryOpUGen` by
  the name of the operator, such as `neg` or `*`, as `UnaryOpUGen.new` and
  `BinaryOpUGen.new` add them. A unary operator runs at the rate of its
  input. A binary operator runs at the highest rate of its inputs, in the
  order demand, audio, control and scalar. A number is at scalar rate.
  `binary_op` applies the folds of `BinaryOpUGen:new1` first. The module
  `operator` gives the special index of each name, and the name of each
  index.
- `madd` multiplies an input and adds to it, as `MulAdd.new` does.
- `variant` adds a variant by control name, from the defaults.
- `finish` collects the constants in the order of creation and of the
  inputs, sorts the UGens as sclang sorts them, and checks the limits.

A `UGenInput` is a number, an `OutputProxy` or an array of them. An
`OutputProxy` gives its rate and the index of its output. A call that fails
leaves the build as it was.

### Multichannel expansion

`ugen`, `unary_op`, `binary_op` and `madd` expand an array as `multiNewList`
expands it in sclang.

```rust
use scsynth_synthdef::{Arg, SynthDefBuilder};

// SynthDef(\pair, { |freqs = #[220, 331], amp = 0.4| Out.ar(0, SinOsc.ar(freqs, 0, amp)) })
let args = vec![Arg::array("freqs", [220.0, 331.0]), Arg::new("amp", 0.4)];
let mut builder = SynthDefBuilder::new("pair", args)?;
let (freqs, amp) = (builder.arguments()[0].clone(), builder.arguments()[1].clone());
let sines = builder.ugen("SinOsc", "ar", &[("freq", freqs.into()), ("mul", amp.into())])?;
builder.ugen("Out", "ar", &[("bus", 0.0.into()), ("channelsArray", sines.into())])?;
let def = builder.finish()?;
let names: Vec<&str> = def.children.iter().map(|ugen| ugen.name.as_str()).collect();
assert_eq!(names, ["Control", "SinOsc", "BinaryOpUGen", "SinOsc", "BinaryOpUGen", "Out"]);
# Ok::<(), Box<dyn std::error::Error>>(())
```

- An array on an argument that expands makes one UGen for each element of
  the longest array. Each array gives its element at the index modulo its
  length, and every other argument stays as it is.
- An element that is itself an array expands again, depth first.
  `SendReply` expands one level only, as its `flop` does, and
  `Wiring::nested_expansion` records it.
- The result is the array of the results, in order: an output for a UGen
  with one output, an array for a UGen with several, and an empty array for
  a UGen with none.
- For `ugen`, the arguments that expand are those that the method gives
  `multiNew`: the arguments of the wire, and the count of the outputs. So
  `In.ar(0, [1, 2])` makes an `In` of one channel and an `In` of two. A
  symbol argument takes one text, and an array argument flattens into the
  inputs.
- `ugen` applies `madd` once, to the whole result, as `Array:madd` does. So
  `SinOsc.ar([440, 442], 0, [0.1, 0.2, 0.3])` makes two `SinOsc` and three
  `*`.
- Each UGen meets the rule of `checkInputs` on its own.

The builder refuses two inputs that sclang takes:

- An empty array where an argument expands, at any depth
  (`BuildError::EmptyInput`). sclang counts it as an input of its UGen, but
  writes no input spec for it (`UGen:numInputs`,
  `Collection:writeInputSpec`). scsynth 3.14.1 refuses that definition with
  "UGen '' not installed.", and `read_def_file` refuses it too. Next to a
  full array, the empty one gives a nil input, which sclang refuses.
- An array inside an array argument, or inside an argument of `SendReply`
  (`BuildError::NestedArray`). sclang expands these in a way that depends on
  the class. `Out.ar(0, [[a, b], [c, d]])` makes `Out(0, a, c)` and
  `Out(0, b, d)`, and `SendReply` takes each inner array of its values
  whole. Below the first level, sclang writes an inner array of `SendReply`
  as one input.

### `madd` and the folds

- `madd(input, mul, add)` takes the first case of `MulAdd:new1` that holds,
  for each instance. `mul` 0 gives `add`. `mul` 1 with `add` 0 gives the
  input, and `mul` -1 with `add` 0 its `neg`. `add` 0 gives `input * mul`,
  `mul` -1 gives `add - input`, and `mul` 1 gives `input + add`.
- If no case holds and `canBeMulAdd` takes the input first, `madd` makes
  that `MulAdd`. If `canBeMulAdd` takes only `mul` first, `madd` makes a
  `MulAdd` with `mul` first. If `canBeMulAdd` takes neither, `madd` makes a
  `*` and a `+`. These operators compute two numbers, so
  `madd(2.0, 3.0, x)` gives `6 + x`.
- A `MulAdd` runs at the least rate by the name of its symbol, as
  `SequenceableCollection:rate` gives it: audio, control, demand, scalar. A
  `BinaryOpUGen` puts demand first.
- `binary_op` folds as `BinaryOpUGen:new1` does, in its order. `* 0` gives
  0. `* 1`, `+ 0`, `- 0` and `/ 1` give the other input. `* -1`, `0 -` and
  `/ -1` give a `neg`. A fold takes numbers only, and the `neg` of a number
  is a number.

### The checks of the inputs

`ugen` checks the inputs as sclang checks them at the end of the build:

- No input is NaN. `add`, `unary_op`, `binary_op` and `madd` check this
  too.
- The rule of `checkInputs` of the class holds. One rule asks for the first
  input at the rate of the UGen. If the UGen runs at audio rate, other
  rules ask for some of its inputs at audio rate. At audio rate,
  `AbstractOut` asks for audio-rate channels.
- A class with a rule of its own is not addable: `T2K`, `Duty`, `TDuty`,
  `BufWr`, `Peak`, `RunningMax` and `RunningMin`.

scsynth does not check the rate of an input. For `Out.ar(0, 0.5)`, it
copies a buffer's length of floats from the one float of the constant
(`IOUGens.cpp:944`). `add` takes the inputs as they come.

## The catalogue

The module `catalog` holds the 419 UGen classes of the core class library
of SuperCollider 3.14.1. sc3-plugins are not in it. `catalog::lookup` gives
a class by name, and `catalog::all` gives all of them, sorted by name.

- A `UGenClass` has its superclasses, whether it is pure or width-first,
  its categories in the help, the rule of its `checkInputs`, and its rate
  methods. `doc_url` gives its page on doc.sccode.org.
- A `RateMethod` is `ar`, `kr`, `ir` or `new`. It has its arguments, each
  with a kind and a default, and an `Addable`.
- `Addable::Yes` holds the `Wiring`: what the method gives `UGen:new1`. It
  holds the rate, the slot of each input, the outputs, the special index,
  whether `mul` and `add` apply, and whether a nested array expands again.
  `Addable::No` holds the reason.

`generator/catalog.scd` measured each method in sclang. It calls the
method twice, each time in a build of its own: once with distinct numbers,
and once with distinct control-rate outputs. Both calls must create
exactly one instance of the class, with the same wiring. Then the method is
addable.
More calls find whether a count or an array sets the number of outputs.
Then, on each argument that expands, an array of two elements must make
two instances, instance i with element i. An array inside an array must
make three. 531 of the 635 methods are addable. Most of the others do more than
`UGen:new1` does:

- The pseudo-UGens, such as `Changed`, create no instance of the class.
- If its input is a number or its lag is 0, the Lag family returns the
  input.
- The delays at audio rate put a `DC` or a `K2A` before their input.
- `LocalBuf` adds a `MaxLocalBufs`, and `Poll` an `Impulse`.
- The control UGens take their special index from the build.
- `UnaryOpUGen` and `BinaryOpUGen` take the name of an operator. Use
  `unary_op` and `binary_op` instead.
- `MulAdd.new` folds, and orders its inputs. Use `madd` instead.

The catalogue gives the page of each class in the help, and not its
summary: the help of SuperCollider is GPL-3, and the crate copies none of
it.

### Methods that `ugen` refuses

`tests/readme.rs` keeps this list equal to the catalogue.

- They create no instance of their class: `Changed.ar`, `Changed.kr`,
  `CompanderD.ar`, `Dstutter.new`, `Dunique.new`, `HilbertFIR.ar`,
  `Lag.ar`, `Lag.kr`, `Lag2.ar`, `Lag2.kr`, `Lag2UD.ar`, `Lag2UD.kr`,
  `Lag3.ar`, `Lag3.kr`, `Lag3UD.ar`, `Lag3UD.kr`, `LagControl.ar`,
  `LagControl.kr`, `LagUD.ar`, `LagUD.kr`, `MulAdd.new`, `Ramp.ar`,
  `Ramp.kr`, `Slew.ar`, `Slew.kr`, `Splay.ar`, `Splay.kr`, `SplayAz.ar`,
  `SplayAz.kr`, `Tap.ar`, `UnpackFFT.new`, `VarLag.ar`, `VarLag.kr`.
- They also create other UGens: `AllpassC.ar`, `AllpassL.ar`,
  `AllpassN.ar`, `BeatTrack.kr`, `BufAllpassC.ar`, `BufAllpassL.ar`,
  `BufAllpassN.ar`, `BufCombC.ar`, `BufCombL.ar`, `BufCombN.ar`,
  `BufDelayC.ar`, `BufDelayL.ar`, `BufDelayN.ar`, `CombC.ar`, `CombL.ar`,
  `CombN.ar`, `DelTapWr.ar`, `DelayC.ar`, `DelayL.ar`, `DelayN.ar`,
  `LinXFade2.ar`, `LinXFade2.kr`, `LocalBuf.new`, `Poll.ar`, `Poll.kr`,
  `Poll.new`.
- An input comes from no argument, or the inputs follow a count:
  `DecodeB2.ar`, `DecodeB2.kr`, `Dwrand.new`, `LocalIn.ar`, `LocalIn.kr`,
  `MaxLocalBufs.new`, `PackFFT.new`, `SendPeakRMS.ar`, `SendPeakRMS.kr`,
  `SetBuf.new`.
- They need arguments that the probe does not make, such as a `Ref`, a
  `Rect`, a `Bus` or an operator, or they do not exist: `BinaryOpUGen.new`,
  `DynKlang.ar`, `DynKlang.kr`, `DynKlank.ar`, `DynKlank.kr`, `InBus.ar`,
  `InBus.kr`, `InRect.ar`, `InRect.kr`, `Klang.ar`, `Klank.ar`,
  `LagControl.ir`, `OffsetOut.kr`, `OutputProxy.new`, `UnaryOpUGen.new`.
- They take their special index from the build: `AudioControl.ar`,
  `Control.ir`, `Control.kr`, `TrigControl.ir`, `TrigControl.kr`.
- They take their rate from their inputs: `Sum3.new`, `Sum4.new`.
- Their class checks its inputs with a rule of its own: `BufWr.ar`,
  `BufWr.kr`, `Duty.ar`, `Duty.kr`, `Peak.ar`, `Peak.kr`, `RunningMax.ar`,
  `RunningMax.kr`, `RunningMin.ar`, `RunningMin.kr`, `T2K.kr`, `TDuty.ar`,
  `TDuty.kr`.

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
  scsynth throws "name too long (> 31 chars)" at 32 (`ReadName`), and the
  acceptance test measures it. The name of a variant includes the name of
  its definition: `<definition>.<variant>`.
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

One limit depends on the server, so the crate does not check it. While the
server runs, scsynth refuses a definition that needs more interconnect
buffers than `-w` gives, 64 by default: "exceeded number of interconnect
buffers." (`SC_GraphDef.cpp:909`). The count also depends on each plugin.

### Traps

The crate does not rule these out, and neither does sclang:

- An array argument shorter than what the UGen reads. For example, a raw
  envelope for `EnvGen` with too few values makes scsynth read past the
  inputs of the unit.
- The inputs of `add`, which take no check but NaN. A UGen with fewer
  inputs than its plugin reads, or with inputs at the wrong rate, reaches
  scsynth as it is.

## Round trips

A file that reads writes back to its own bytes, with the writer of its
version: `write_def_file` for version 2, and `write_def_file_old` for
version 1. Each float keeps its bits, NaN included. Version 0 has no
writer. `tests/golden.rs` checks the round trip of every golden file.

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
- An empty array where an argument expands, and a nested array in an array
  argument or in `SendReply`: the builder refuses them (see Multichannel
  expansion).
- An array of symbols: sclang expands a symbol argument too, as the command
  names of `SendReply`. `ArgValue` holds one text.
- Two numbers: `unary_op` and `binary_op` make a UGen of them, as
  `UnaryOpUGen.new` and `BinaryOpUGen.new` do. The operators of sclang
  compute them instead, as `2 * 3` gives 6. The crate cannot copy that.
  `2 < 3` is `true`, which no UGen takes. When sclang builds, `rand` draws.
  sclang gives nan for `-2 ** 0.5`, and the `pow` of scsynth keeps the
  sign.
- `mul` and `add` on a UGen without outputs, such as `AnalogOut`: sclang
  multiplies an output that the UGen does not have, and `optimizeGraph`
  removes the product. `ugen` refuses a value other than 1 and 0.
- A 0 channel of `Out.ar`, `XOut.ar` or `LocalOut.ar`, such as `binary_op`
  gives for `* 0`: sclang puts a `DC` in its place. `ugen` keeps the 0, and
  the rule of `AbstractOut` refuses it.

## Left out

These are not in version 0.2, in the order of their value for sc-app3:

1. nested arrays in array arguments, as each class expands them
2. `optimizeGraph`, and `addCopiesIfNeeded` for PV chains
3. what rate methods do besides `new1`: the Lag family,
   `asAudioRateInput`, `LocalBuf`, `Poll` and the zeros of `Out`
4. `NamedControl`, `Control.names`, `SynthDef.wrap`, `prependArgs` and the
   specs of the metadata
5. `Env`, and the pseudo-UGens such as `Mix`, `Splay` and `SoundIn`
6. the rules of `checkInputs` that classes define for themselves
7. sc3-plugins

Also left out: the view of `SynthDesc` (its inputs, outputs and metadata),
and the count of interconnect buffers.

## Tests

`cargo test --all-features` runs every test but the acceptance test. It
needs no SuperCollider. `tests/readme.rs` checks the list of methods that
`ugen` refuses.

The golden files in `tests/fixtures` are definitions that sclang 3.14.1
built and wrote. Each one reads, and writes back to its own bytes.
`generator/fixtures.scd` writes them again:

```sh
sclang -l "$PWD/generator/sclang_conf.yaml" "$PWD/generator/fixtures.scd"
```

- `generator/sclang_conf.yaml` compiles the core class library of
  `/Applications/SuperCollider.app` alone, without the Extensions folder and
  without quarks. For another installation, change its path.
- For a graph of 0.1 or 0.2: if `finishBuild` adds, removes or rewires a
  UGen, the script fails.
- For the graphs of 0.3: if `optimizeGraph` rewrites a definition of a file,
  `tests/fixtures/before_optimize` holds a second file. The script builds it
  with `finishBuild` without `optimizeGraph`, as the crate builds.
- `tests/fixtures/arithmetic.json` holds operands and results of the
  operators that sclang computes on Floats, as the bits of each f64.
- `~/.bin/sclang` changes the directory, so the paths must be absolute.

`generator/catalog.json` holds the catalogue and the tables of the
operators. The operators come from `Opcodes.h` and `initSpecialSelectors`
in the sources of SuperCollider, at the tag `Version-3.14.1`, and sclang
must agree with each special index. `generator/catalog.scd` writes the
JSON again, in two seconds:

```sh
SC_SOURCE=~/Projects/supercollider \
sclang -l "$PWD/generator/sclang_conf.yaml" "$PWD/generator/catalog.scd"
```

`tests/render.rs` renders the JSON into `src/operator/generated.rs` and
`src/catalog/generated.rs`. If a committed file differs, the test fails.
With `SCSYNTH_SYNTHDEF_BLESS=1` set, it writes the file instead.

`tests/scsynth.rs` sends definitions of the crate to scsynth 3.14.1. It is
ignored by default:

```sh
cargo test --test scsynth -- --ignored
```

- scsynth runs in NRT mode: it opens no audio device and no port. When it
  exits, its output is complete. `SCSYNTH` names the scsynth, and
  `SCSYNTH_PLUGINS` its plugins. Both default to the bundle of
  `/Applications/SuperCollider.app`. `-U` keeps the plugins of the
  Extensions folder out.
- scsynth must load every golden graph, built by the crate, without an
  exception. A last definition with a name too long shows that it read
  the whole score.
- A parameter name and a variant name of 31 bytes load. A UGen name of 31
  bytes reaches the message `UGen '<name>' not installed.`. Each of the
  three at 32 bytes throws "name too long (> 31 chars)".
- The test only loads definitions. In NRT mode, scsynth 3.14.1 crashes on
  `/s_new` of a definition with an `AudioControl`.

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
