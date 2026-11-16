# scsynth-synthdef

The synth definitions of SuperCollider, for Rust. This crate compiles a
graph of UGens the way sclang compiles a `SynthDef`, into the
[SynthDef file format](https://doc.sccode.org/Reference/Synth-Definition-File-Format.html)
(`SCgf`). scsynth loads these bytes with `/d_recv`. The crate reads them
back, strictly. It carries a catalogue of the UGens of the core class
library. With the feature `sc3-plugins`, it carries one of sc3-plugins too
(see [sc3-plugins](#sc3-plugins)).

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
- `SynthDef::wire_buffers` counts the wire buffers that scsynth gives a
  definition (see [Wire buffers](#wire-buffers)).

The versions differ in two places:

- Version 2 stores counts and indices as int32, and version 1 as int16.
- Version 0 is version 1 without the variants. scsynth reads the count of
  variants only from version 1 on (`SC_GraphDef.cpp:505`).

The count of definitions and the count of variants are an int16 in every
version.

## Building

A `SynthDefBuilder` is the `SynthDef` of sclang during a build. It builds
as `SynthDef.new` does.

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
- `wrap`, `control_names`, and `ugen` with `NamedControl` and the classes
  of controls add controls after the arguments (see
  [Controls inside the graph](#controls-inside-the-graph)).
- `env` makes an envelope by a class method of `Env`, such as `perc`.
  `EnvGen` and `IEnvGen` take it as their `envelope` (see
  [Envelopes](#envelopes)).
- `ugen` adds a UGen by its class name and the selector of a rate method,
  `ar`, `kr`, `ir`, `tr` or `new`, with the arguments by name. The catalogue
  gives the order of the arguments, their defaults, the wiring and the
  rule of `checkInputs`. An `ArgValue` is an input, a text or a boolean. A
  text goes to a symbol argument, such as the command name of `SendReply`.
  A boolean goes to an argument such as `wrap` of `SelectXFocus`. A single
  input goes where the method takes an array, as `asArray` makes it. An array
  expands the UGen, and `mul` and `add` go through `madd`.
- At audio rate, a delay passes its input through `asAudioRateInput`
  before the expansion. A number becomes a `DC`, and an output at another
  rate a `K2A`. `LinXFade2` multiplies its result by `level`, after the
  expansion.
- Some methods do more than `UGen:new1`, and the crate copies them (see
  [Methods that do more than `new1`](#methods-that-do-more-than-new1)).
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
- `variant` adds a variant by control name, from the defaults. A name that
  two controls share sets the last of them, as the map of names of sclang
  does.
- `finish` copies the PV chains and rewrites the graph, as `finishBuild`
  does (see
  [`optimizeGraph` and the copies of PV chains](#optimizegraph-and-the-copies-of-pv-chains)).
  Then it collects the constants in the order of the children and of the
  inputs. Last, it sorts the UGens as sclang sorts them, and checks the
  limits.

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
- An array among the elements of an array argument expands as its class
  expands it, and `Wiring::array_expansion` records how:
  - `Elements`: the method hands each element to `multiNewList`, so an
    element that is an array expands the UGen with the other arguments.
    `Out.ar(0, [[a, b], c])` makes `Out(0, a, c)` and `Out(0, b, c)`, and so
    do the methods of `BufWr`, `Select`, `Demand` and `Dseq`, for example.
  - `Flop`: with an array among the values of `SendReply`, each element
    gives the values of one instance, one level down.
  - `Refs`: if the first element of the envelope of `EnvGen` is an array,
    each element is one envelope.
  - `IEnvGen` keeps the arrays of its envelope among its inputs, which the
    builder refuses.
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
- An array that stays among the inputs of a UGen
  (`BuildError::NestedArray`). `Array:isValidUGenInput` answers true, so
  sclang counts such an array as one input. Then it writes an input spec for
  each of its elements, and scsynth misreads the file. Such an array comes
  from two levels down in the values of `SendReply`, or in an argument of
  it. The envelope of `EnvGen` keeps one after a first element that is not
  an array. The envelope of `IEnvGen` keeps every array inside.

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
- Four classes have a rule of their own, and their subclasses inherit it.
  At audio rate, `BufWr` asks for its phase and its channels at audio
  rate. After a `dur` at demand rate, `Duty` and `TDuty` ask for a `reset`
  at demand rate, at scalar rate or at their own rate.
- At control rate, `Peak`, `RunningMax` and `RunningMin` also take an input
  at audio rate. `T2K` asks for an input at audio rate.

scsynth does not check the rate of an input. For `Out.ar(0, 0.5)`, it
copies a buffer's length of floats from the one float of the constant
(`IOUGens.cpp:944`). `add` takes the inputs as they come.

### Pseudo-UGens

A pseudo-UGen is a class whose rate method builds other UGens, such as
`LinLin.kr`. scsynth has no unit of its name. `ugen` adds a pseudo-UGen by
its class name, as it adds any class of the catalogue. It gives back what
the method gives in sclang: a number, an output, or an array of them.

```rust
use scsynth_synthdef::{SynthDefBuilder, UGenInput};

// LinLin.kr(0.5, 0, 1, 10, 20): with numbers, sclang computes the result.
let mut builder = SynthDefBuilder::new("lin", vec![])?;
let args = [
    ("in", 0.5.into()),
    ("srclo", 0.0.into()),
    ("srchi", 1.0.into()),
    ("dstlo", 10.0.into()),
    ("dsthi", 20.0.into()),
];
assert_eq!(builder.ugen("LinLin", "kr", &args)?, UGenInput::Number(15.0));
# Ok::<(), Box<dyn std::error::Error>>(())
```

```rust
use scsynth_synthdef::{SynthDefBuilder, UGenInput};

// SynthDef(\splay, { Out.ar(0, Splay.ar(SinOsc.ar([220, 330]))) })
let mut builder = SynthDefBuilder::new("splay", vec![])?;
let freqs = UGenInput::Array(vec![220.0.into(), 330.0.into()]);
let voices = builder.ugen("SinOsc", "ar", &[("freq", freqs.into())])?;
let stereo = builder.ugen("Splay", "ar", &[("inArray", voices.into())])?;
builder.ugen("Out", "ar", &[("bus", 0.0.into()), ("channelsArray", stereo.into())])?;
let def = builder.finish()?;
let names: Vec<&str> = def.children.iter().map(|ugen| ugen.name.as_str()).collect();
let ops = ["BinaryOpUGen"; 4];
assert_eq!(names[..4], ["SinOsc", "Pan2", "SinOsc", "Pan2"]);
assert_eq!(names[4..8], ops);
assert_eq!(names[8], "Out");
# Ok::<(), Box<dyn std::error::Error>>(())
```

The two `+` sum the channels of the two `Pan2`. The two `*` apply the level
that `levelComp` gives two inputs at audio rate: the square root of 1/2.

- A body of the crate follows the sclang code of each method. It adds the
  UGens in the order in which sclang adds them, so the topological sort
  gives the order of sclang.
- A body computes with its arguments as the language does. Two numbers
  give a number, in f64, as sclang computes its Floats. An array goes
  element by element. Anything else goes to `binary_op` and `unary_op`.
- `tests/fixtures/arithmetic.json` tests each operator against sclang
  3.14.1 for macOS on arm64. That sclang computes `%` and `fold2` of Floats
  with one fused multiply-add, and so does the crate.
- The crate builds `BHiPass4`, `BLowPass4`, `Changed`, `CompanderD`,
  `Dstutter`, `DynKlang`, `DynKlank`, `HilbertFIR`, `LinLin`, `LinSelectX`,
  `Mix`, `MultiTap` and `NumChannels`.
- It also builds `PingPong`, `PMOsc`, `SelectX`, `SelectXFocus`, `Silent`,
  `SoundIn`, `Splay`, `SplayAz`, `TChoose`, `TWChoose`, `Tap` and
  `UnpackFFT`.
- `Splay`, `DynKlank` and `DynKlang` expand before their body, as their
  `multiNew` or `multiNewList` does. `Splay` expands over each element of
  `inArray` too, so an array of stereo pairs makes one Splay for each
  channel.
- The spec of `DynKlank`, `DynKlang`, `Klank` and `Klang` is an array of up
  to three arrays, as `[freqs, amps, times]`. A missing array takes its
  default. Where sclang takes nil in the middle, give the default, such as
  `[1.0]`. `Klank` and `Klang` are real UGens: the triplets of the spec,
  with their parts wrapped to the longest, end their inputs.
- `PingPong` takes an array of channels and a whole `rotate`, as sclang
  does. It gives back the delayed channels.
- `Sum3.new` and `Sum4.new` have a body too. An input of 0 folds the UGen
  into a smaller sum. Then the UGen takes the least rate of its inputs by
  name. It orders its inputs as the insertion sort of sclang does, which
  reverses inputs of one rate.
- `Mix` clumps its array by 4 into `Sum4`, `Sum3` or `+`, and mixes the
  mixes again. Two numbers give a number, as `Mix([1, 2])` gives 3, but
  `Mix([1, 2, 3])` gives a `Sum3` at scalar rate. `BuildError::UnsupportedRate`
  refuses `Mix.ar` and `Mix.kr` of a mix at demand rate.
- A pseudo-UGen fails as the UGens that it adds fail. A call that fails
  adds no UGen.

### Methods that do more than `new1`

Some rate methods of real UGens do more than `UGen:new1`. The crate copies
what each one does in sclang.

- If its input runs at scalar rate, as a number does, the Lag family gives
  back `in.madd(mul, add)` in place of a UGen. If `lagTime` is 0, `Lag`,
  `Lag2`, `Lag3` and `Ramp` also give it back. `LagUD`, `Lag2UD`, `Lag3UD`
  and `Slew` do not. For a number, `madd` computes `(in * mul) + add`.
- The test runs before the expansion. So `Lag.kr([control, 2])` makes two
  `Lag`, and the second breaks the rule of `checkInputs`, as in sclang.
- At audio rate, `Out`, `ReplaceOut`, `OffsetOut`, `LocalOut` and `XOut`
  put silence in place of each 0 channel, -0.0 too. The zeros of the top
  level of the array share one `DC.ar(0)`, from the body of `Silent`.
- The first `LocalBuf` of a build adds its `MaxLocalBufs`, and each
  `LocalBuf` adds 1 to its count. `finish` writes the count. Each
  `LocalBuf` is width-first, and takes the `MaxLocalBufs` as its last
  input.
- `BeatTrack.kr` puts an `FFT` of a `LocalBuf` in front of a chain that
  comes from no `FFT`. The size of the buffer is
  `if(SampleRate.ir > 48000, 2048, 1024)`, with the arithmetic of
  `UGen:if`.
- `LocalIn` has as many outputs as its count. Its default repeats to fill
  the count.
- `Poll` gives back its input, not itself. A number trigger becomes an
  `Impulse` at the rate of the `Poll`. Without a label, the label names the
  class of the input, as "UGen(SinOsc)". `Poll.new` takes the rate of each
  element of `in` before the expansion. So in
  `Poll.new(10, [[SinOsc.ar, SinOsc.kr], x])`, both elements of the inner
  array get a `Poll` at audio rate.
- `DemandEnvGen.ar` takes its `gate` and its `reset` to audio rate
  together. If one of them runs at audio rate, `K2A.ar` takes the other
  one. The rate of an array is the least of its rates by name, so
  `[Impulse.ar(1), 1]` is at audio rate.
- `PackFFT.new` writes the size of `magsphases` before its values. A nil
  `tobin` is `bufsize / 2`, so an output as `bufsize` adds the `/` before
  the `PackFFT`.
- The codes of a symbol and of a label are its bytes as `Char:ascii` gives
  them in sclang, from -128 to 127.

### `optimizeGraph` and the copies of PV chains

`finish` runs `addCopiesIfNeeded` and `optimizeGraph`, as `finishBuild`
does, before it collects the constants. So the constants follow the order
of the rewritten graph.

- A PV chain that two or more width-first UGens read gets copies, right
  after the chain. Each slot of each reader but the last gets one: a
  `BufFrames`, a `LocalBuf` and a `PV_Copy`. Each `LocalBuf` counts in the
  `MaxLocalBufs` of the build. `fft_size` of the catalogue tells how a
  chain finds the size of its FFT.
- `optimizeGraph` visits each UGen in the order of the children. A pure
  UGen that nothing reads goes, and each of its direct inputs gets another
  visit. An output of a `MultiOutUGen` comes through an `OutputProxy`, so
  the visit stops there: in `DC.ar(1) * 2`, the `*` goes, and the `DC`
  stays. `optimize` of the catalogue tells what each class does.
- A `+` becomes a `Sum3`, a `Sum4`, a `MulAdd` or a `-`, the first that
  holds. A `-` of a `neg` becomes a `+`. A rewrite fuses only an input that
  one UGen reads. So `osc * 0.5 + 0.25` becomes one `MulAdd`.
- The crate copies the bookkeeping of sclang, with its quirks. The array of
  the readers of a UGen has the room of sclang, so it is full at 4, 8, 16
  and more readers. On a full array, an update goes missing. The order of
  the inputs of a removed UGen decides which inputs fuse later.

`BuildError::FinishBuild` refuses a graph where `finishBuild` of sclang
3.14.1 throws or writes a broken definition:

- A copy needs a `LocalBuf`, and the definition has none: sclang drops
  UGens. Give the `FFT` a `LocalBuf`.
- A copy of a chain of `PackFFT`, or of a chain without `fftSize`, such as
  one of `FFTTrigger`: sclang throws.
- A rewrite folds into a number or into one of its inputs, as
  `x + neg(0)`, where `unary_op` made the `neg` of a number: sclang throws,
  or writes a UGen twice.
- After an update that goes missing, `optimizeGraph` removes a UGen that
  another UGen reads: sclang drops UGens. So `x * 2 + 1`, with three more
  readers of `x` that nothing reads, gives a definition with no UGens.

### Controls inside the graph

A graph function can make controls after its arguments. The crate builds
them as sclang 3.14.1 does, and each control UGen takes the next
parameters of the build.

```rust
use scsynth_synthdef::{Arg, SynthDefBuilder};

// SynthDef(\named, { |amp = 0.1|
//     var freq = \freq.kr(440, 0.1);
//     Out.ar(0, SynthDef.wrap({ |pan = 0| Pan2.ar(SinOsc.ar(freq), pan) }) * amp)
// })
let mut builder = SynthDefBuilder::new("named", vec![Arg::new("amp", 0.1)])?;
let amp = builder.arguments()[0].clone();
let freq = builder.ugen(
    "NamedControl",
    "kr",
    &[("name", "freq".into()), ("values", 440.0.into()), ("lags", 0.1.into())],
)?;
let sound = builder.wrap(vec![Arg::new("pan", 0.0)], |builder, inputs| {
    let sine = builder.ugen("SinOsc", "ar", &[("freq", freq.into())])?;
    builder.ugen("Pan2", "ar", &[("in", sine.into()), ("pos", inputs[0].clone().into())])
})?;
let sound = builder.binary_op("*", sound, amp)?;
builder.ugen("Out", "ar", &[("bus", 0.0.into()), ("channelsArray", sound.into())])?;
let def = builder.finish()?;
let names: Vec<&str> = def.control_names.iter().map(|name| name.name.as_str()).collect();
assert_eq!(names, ["amp", "freq", "pan"]);
assert_eq!(def.controls, [0.1, 440.0, 0.0]);
# Ok::<(), Box<dyn std::error::Error>>(())
```

- `NamedControl.ar`, `kr`, `ir`, `tr` and `new` add a named control, as
  `\freq.kr(440)` does. The first use of a name makes its control UGen,
  and a later use gives back the same control.
- A name whose second byte is `_` has a prefix: its first byte. The first
  match gives the rate: audio (the prefix `a` or `ar`), trigger (`t` or
  `tr`), scalar (`i` or `ir`), else control. The arguments of `new` try
  scalar first.
- Each 0 among the lags becomes nil, so a lag of 0 means no lag. Lags at
  scalar rate, such as a number, are fixed, and so are all lags with
  `fixedLag`. Fixed lags make a `LagControl`. Other lags add a `Lag` at
  each use, at the rate of the control.
- The order of the uses counts. A first use with a number lag makes a
  `LagControl`, which a later use without a lag reads. A first use without
  a lag, then a number lag, gives a `Control` and a `Lag`.
- A prefix drops fixed lags, so `\x_freq.kr(440, 0.1)` has no lag.
  `NamedControl.ir` and `NamedControl.tr` with a number lag give a
  `LagControl` at control rate.
- The values can nest. The control takes them flat, and the result takes
  their shape. A spec without values gives the default of the spec of that
  name in `catalog::spec`.
- `control_names` names the next parameters, as `Control.names` does. Then
  `ugen` adds `Control`, `TrigControl`, `AudioControl` or `LagControl` by
  name. If the last name has no default, a `Control` or a `TrigControl`
  gives it its values.
- `LagControl.kr` takes the values in groups of 16, and gives back an
  array. `LagControl.ar` is an `AudioControl` and its `Lag`s.
- `wrap` adds the controls of the arguments of a function inside the graph
  function, as `SynthDef.wrap` does. Then it calls a closure with their
  inputs. The controls take the groups of rate of `new`. For
  `prependArgs`, the closure takes their values from its caller, and `new`
  and `wrap` get only the arguments that become controls.
- If the metadata of a definition has specs, sclang gives an argument
  without a default the default of the spec of its name. Give that default
  to `Arg::new`.
- Two controls can share a name, such as an argument and a `NamedControl`.
  The crate writes both entries, as sclang does (see [Traps](#traps)).

`BuildError::BadControl` refuses a control that sclang throws on:

- A later use of a `NamedControl` that gives other values, another rate,
  or other fixed lags than the first use.
- An unknown spec on a `NamedControl` without values.
- A 0 among lags that reach a UGen. A prefix keeps fixed lags from the
  UGens.
- Values and lags of `LagControl.kr` that differ in size: sclang gives nil,
  and its reader throws.

### Envelopes

`env` makes an envelope by a class method of `Env`, with the arguments by
name and the defaults of sclang 3.14.1. `EnvGen` and `IEnvGen` take the
`Env` as their `envelope`.

```rust
use scsynth_synthdef::{Arg, SynthDefBuilder};

// SynthDef(\adsr, { |gate = 1|
//     Out.ar(0, SinOsc.ar * EnvGen.kr(Env.adsr(sustainLevel: 0.7), gate, doneAction: 2))
// })
let mut builder = SynthDefBuilder::new("adsr", vec![Arg::new("gate", 1.0)])?;
let gate = builder.arguments()[0].clone();
let sine = builder.ugen("SinOsc", "ar", &[])?;
let env = builder.env("adsr", &[("sustainLevel", 0.7.into())])?;
assert_eq!(env.release_node, Some(2.0));
let level = builder.ugen(
    "EnvGen",
    "kr",
    &[("envelope", env.into()), ("gate", gate.into()), ("doneAction", 2.0.into())],
)?;
let sound = builder.binary_op("*", sine, level)?;
builder.ugen("Out", "ar", &[("bus", 0.0.into()), ("channelsArray", sound.into())])?;
assert_eq!(builder.finish()?.children.len(), 5);
# Ok::<(), Box<dyn std::error::Error>>(())
```

- The constructors are `new`, `newClear`, `triangle`, `sine`, `perc`,
  `linen`, `xyc`, `pairs`, `step`, `cutoff`, `dadsr`, `adsr`, `asr` and
  `circle`. Each does its math at the time of the `Env`. So `adsr` with
  outputs as `peakLevel` and `sustainLevel` adds their `*` before the UGens
  that follow. `circle` adds `Impulse`, `Delay1` and `Latch`.
- A `Curve` is a shape by name, a value or an array. The shapes are `step`,
  `lin`, `exp`, `sin`, `wel`, `sqr`, `cub` and `hold`, and long names such
  as `linear`. A value is the shape 5, with this curvature. An array gives
  one curve per segment, in turn, or one per channel. A `curve` argument
  also takes a text as a shape, and an input as values.
- `Env::as_array` gives the array that `EnvGen` reads, and
  `Env::as_multichannel_array` keeps the outer array of one channel.
  `env_array_for_interpolation` gives the form of `IEnvGen`. When a time is
  an output, its sum of the times adds UGens, as `times.sum` does.
  `Env::delay` puts a delay before the envelope. An array among the levels
  or the curves makes channels.
- `EnvGen` takes an `Env` through `as_array`. `IEnvGen` takes its channels,
  and several channels expand with the index, as `IEnvGen.convertEnv`
  does. For `Env.kr` and `Env.ar` of sclang, add `EnvGen.kr` and
  `EnvGen.ar` of the `Env` with `ugen`.
- `VarLag.ar` and `VarLag.kr` give back the input and its `madd` for an
  input at scalar rate, or a `time` of 0. A curve of 1 makes a real
  `VarLag`. Any other curve makes the `EnvGen` of an envelope of two
  levels, which `Changed` and `Impulse` trigger.
- `EnvGate.new` makes `\gate.kr(1)` and `\fadeTime.kr(0.02)` for the
  arguments that are nil, as `NamedControl` makes them. Then it makes the
  `EnvGen` of its envelope.

`BuildError::BadEnv` refuses an envelope that sclang throws on:

- An envelope with no levels.
- A shape that sclang does not know, also as the `warp` of `VarLag`.
- Levels and times of `Env.step` that differ in size.

### The `pvcalc` methods

`pvcalc`, `pvcalc2` and `pvcollect` copy the methods of `PV_ChainUGen`
that take a function. Each unpacks the bins of a chain from `frombin` to
`tobin`, as `UnpackFFT` does, and calls a closure in place of `func`. Then
it packs what the closure gives into the chain again, as `PackFFT` does.
The arguments come by name, with the defaults of sclang 3.14.1.

```rust
use scsynth_synthdef::SynthDefBuilder;

// SynthDef(\bins, { var chain = FFT(LocalBuf(8), SinOsc.ar);
//     Out.ar(0, IFFT(chain.pvcollect(8, { |mag, phase, bin| mag * bin }))) })
let mut builder = SynthDefBuilder::new("bins", vec![])?;
let buffer = builder.ugen("LocalBuf", "new", &[("numFrames", 8.0.into())])?;
let sine = builder.ugen("SinOsc", "ar", &[])?;
let chain = builder.ugen("FFT", "new", &[("buffer", buffer.into()), ("in", sine.into())])?;
let packed = builder.pvcollect(chain, &[("numframes", 8.0.into())], |b, magnitude, _, bin, _| {
    b.binary_op("*", magnitude, bin)
})?;
let sound = builder.ugen("IFFT", "new", &[("buffer", packed.into())])?;
builder.ugen("Out", "ar", &[("bus", 0.0.into()), ("channelsArray", sound.into())])?;
let def = builder.finish()?;
assert!(def.children.iter().any(|ugen| ugen.name == "PackFFT"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

- `pvcalc` gives its closure the array of the magnitudes and the array of
  the phases. The size of the result decides what `PackFFT` takes, as in
  sclang. An array of one value gets the phases after it, and an array of
  two is the magnitudes and the phases. Anything else, a UGen included, is
  the magnitudes, with the phases of the chain.
- So two bins with their magnitudes alone count as the magnitudes and the
  phases. `UnpackFFT` keeps `tobin` at most at `numframes / 2`, but
  `PackFFT` takes it as it is.
- `pvcalc2` takes `chain2` by name. Its closure gets the magnitudes and the
  phases of the chain, then those of `chain2`.
- `pvcollect` calls its closure for each bin, with its magnitude, its phase,
  its bin and its index, the last two as numbers. A result of one value
  gets the phase of its bin, and an array of more values stays.

A `BuildError::WrongKind` refuses a chain that is not one output of a
`PV_ChainUGen`, because sclang understands the methods on no other value.
It also refuses `numframes`, `frombin` and `tobin` as outputs, where sclang
throws, and a value of `func` by name.

### The methods of `UGen`

`send` sends an instance method of `UGen` that builds a graph, such as
`range` or `lag`, to a receiver: an output, an array or a number. The
arguments come by name, with the defaults of the method of `UGen` in
sclang 3.14.1. `send_env` maps the levels of an `Env` in the same way.

```rust
use scsynth_synthdef::SynthDefBuilder;

// SynthDef(\sweep, { Out.ar(0, SinOsc.ar(LFPulse.kr(2).range(200, 800).lag(0.05)) * 0.1) })
let mut builder = SynthDefBuilder::new("sweep", vec![])?;
let pulse = builder.ugen("LFPulse", "kr", &[("freq", 2.0.into())])?;
let freq = builder.send(pulse, "range", &[("lo", 200.0.into()), ("hi", 800.0.into())])?;
let freq = builder.send(freq, "lag", &[("t1", 0.05.into())])?;
let sine = builder.ugen("SinOsc", "ar", &[("freq", freq.into())])?;
let sound = builder.binary_op("*", sine, 0.1)?;
builder.ugen("Out", "ar", &[("bus", 0.0.into()), ("channelsArray", sound.into())])?;
let def = builder.finish()?;
let names: Vec<&str> = def.children.iter().map(|ugen| ugen.name.as_str()).collect();
assert_eq!(names, ["LFPulse", "MulAdd", "Lag", "SinOsc", "BinaryOpUGen", "Out"]);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`LFPulse` is unipolar, so `range` maps 0 to 200 and 1 to 800: a `MulAdd`
of 600 and 200.

- The 42 methods are `range`, `exprange`, `curverange`, `unipolar` and
  `bipolar`. Then come the maps `linlin`, `linexp`, `explin`, `expexp`,
  `lincurve`, `curvelin`, `bilin` and `prune`. Then come `clip`, `fold` and
  `wrap`, and the lag family: `lag`, `lag2`, `lag3`, `lagud`, `lag2ud`,
  `lag3ud`, `varlag` and `slew`. Then `blend`, `minNyquist`, `degrad`,
  `raddeg`, `moddif`, `sanitize`, `poll`, `dpoll` and `checkBadValues`.
  Last come `snap`, `softRound`, `binaryValue`, `isPositive`,
  `isNegative`, `isStrictlyPositive`, `|==|`, `degreeToKey` and `if`.
- The call passes every argument, in its order, to the method that the
  receiver answers. So it is the call of sclang with every argument by
  position.
- An output gets the method of the class of its UGen. `DUGen`, the
  superclass of `Dseq` and the other demand UGens, has its own `linlin`,
  `linexp`, `explin` and `expexp`, which compute with operators. `LFGauss`
  has its own `range`, from the least value of its width. A message that
  a method sends, as `exprange` sends `linexp`, goes to the class of the
  receiver too.
- `range`, `exprange` and `curverange` read the `signalRange` of the
  class. 16 classes are unipolar, such as `LFPulse`, `Impulse` and
  `Dust`. An output of a UGen with several outputs is an `OutputProxy`,
  which is bipolar.
- The lag family calls `multiNew` at the rate of the receiver, so it skips
  the passthrough of the rate methods. `lag(0)` adds its `Lag`, and a
  receiver at scalar or demand rate gives a `Lag` at that rate. `varlag`
  builds the `VarLag` of `VarLag.kr` in the same way.
- `clip`, `fold` and `wrap` take `ir` at scalar rate. At demand rate,
  `clip` is `max(lo, min(hi, x))`. `checkBadValues` gives back the
  receiver, and adds nothing at scalar or demand rate, as in sclang.
- An array gets the method of `Array`. Most methods hand the message to
  each element, as `multiChannelPerform` does. An array among the
  arguments spreads over the elements, and the shorter array wraps.
  `degrad`, `raddeg`, the signs and `degreeToKey` send the same arguments
  to each element. `minNyquist` adds one `SampleRate`. `poll` and `dpoll`
  call `Poll` and `Dpoll` once with the whole array. So the arrays among
  their arguments lace with the elements in the same way. If `label` is
  nil, they label each element `UGen Array [i]`.
- A number gets the method of `Float`. It computes a number where it can,
  as `0.25.linlin(0, 1, 200, 800)` gives 350. The lag family gives the
  number itself. An output among the arguments adds UGens, as in sclang.

`BuildError::BadMethod` refuses a message that sclang throws on:

- A receiver that does not understand the message. A number does not
  understand `range`, `exprange`, `curverange`, `unipolar`, `bipolar`,
  `prune`, `minNyquist`, `sanitize`, `dpoll`, `checkBadValues` or `if`. An
  array does not understand `snap`, `softRound`, `binaryValue` or `if`.
  An empty array is refused too.
- `fold`, `wrap` and `blend` at demand rate: sclang does not implement
  them.
- The test of an `if` on an output, as in `linlin` of a number with an
  output as `inMin`: "Non Boolean in test."
- A Boolean: the signs of a number, and `|==|` of two numbers.
- An output as a bound of `clip`, `fold` or `wrap` of a number, whose
  primitives take numbers.
- A nil `clip` of `expexp` on a `DUGen`, which divides it by `inMin`.

It also refuses the definitions that sclang writes without their `Out`.
At scalar or demand rate, `methodSelectorForRate` names a method that the
class does not have as a UGen. So `sanitize`, `bilin`, `lincurve` and
`curvelin` with a curve that is an output, and `blend` at scalar rate,
build no UGen in sclang.

`XFade2` takes two inputs at audio rate. So `blend` of an input at audio
rate and one at another rate gives `BuildError::CheckInputs`, as sclang
throws. `varlag` of a receiver at scalar rate gives it too, from the
`HPZ1` of its `Changed.kr`.

`send_env` gives `range`, `exprange` and `curverange` of an `Env`: a copy
of the envelope, with its levels mapped by `linlin`, `linexp` or
`lincurve` from the least level to the greatest. `BuildError::BadEnv`
refuses an output among the levels, which `minItem` and `maxItem` cannot
compare.

## The catalogue

The module `catalog` holds the 419 UGen classes of the core class library
of SuperCollider 3.14.1. The classes of sc3-plugins are in a module of
their own (see [sc3-plugins](#sc3-plugins)). `catalog::lookup` gives
a class by name, and `catalog::all` gives all of them, sorted by name.
`catalog::pseudo_lookup` and `catalog::pseudo_all` give the pseudo-UGen
classes outside `UGen` that the crate builds, such as `LinLin`.

- A `UGenClass` has its superclasses, whether it is pure or width-first,
  and its categories in the help. It also has the rule of its
  `checkInputs`, its `optimize`, its `fft_size`, its `cant_alias` and its
  rate methods. `optimize` is what `optimizeGraph` does with a UGen of the
  class, and `fft_size` is how `fftSize` finds the FFT of a PV chain.
  `cant_alias` is the flag of its plugin that the count of
  [wire buffers](#wire-buffers) reads. `doc_url` gives its page on
  doc.sccode.org.
- A `PseudoClass` has its categories in the help and its rate methods.
- `catalog::spec` and `catalog::specs` give the 23 `ControlSpec`s of
  `Spec.specs`: the range, the warp, the step, the default and the units
  of each name, such as `freq`.
- `catalog::env_method` and `catalog::env_methods` give the class methods
  of `Env` that make an envelope, with their arguments.
  `UGenClass::convert_env` tells how a class takes an `Env`.
- `catalog::pv_method` and `catalog::pv_methods` give the instance methods
  of `PV_ChainUGen` that take a function, with their arguments.
  `ArgKind::Function` marks `func`.
- `catalog::ugen_method` and `catalog::ugen_methods` give the instance
  methods of `UGen` that build graphs, with their arguments and the
  defaults of `UGen`. A `UGenMethod` also names the classes whose methods
  a Float and an `Array` answer, and the subclasses of `UGen` with a
  method of their own. `UGenClass::signal_range` is the `signalRange` of
  the class.
- `catalog::env_range` and `catalog::env_ranges` give `range`, `exprange`
  and `curverange` of `Env`, with their arguments.
- A `RateMethod` is a rate method, such as `ar`, `kr`, `ir`, `tr` or
  `new`, or one of the methods above. It has its arguments, each with a
  kind and a default, and an `Addable`.
- `Addable::Yes` holds the `Wiring`: what the method gives `UGen:new1`. It
  holds the rate, the slot of each input, the outputs, the special index,
  whether `mul` and `add` apply, and whether a nested array expands again.
  It also holds the argument that goes through `asAudioRateInput`, the
  arguments that go to audio rate together, and whether `level` multiplies
  the result. It holds the `Passthrough` of the
  Lag family, the argument whose zeros become silence, and the
  `ArrayExpansion` of the array argument.
  `Addable::Pseudo` marks a method that a body of the crate builds (see
  [Pseudo-UGens](#pseudo-ugens) and
  [Methods that do more than `new1`](#methods-that-do-more-than-new1)).
  `Addable::No` holds the reason.

`generator/catalog.scd` measured each method in sclang. It calls the
method twice, each time in a build of its own: once with distinct numbers,
and once with distinct control-rate outputs. Both calls must create
exactly one instance of the class, with the same wiring. Then the method is
addable. The input of a delay at audio rate takes distinct audio-rate
outputs instead, and `level` of `LinXFade2` takes 1. The input of the Lag
family takes a control-rate output, so that the method makes an instance.
More calls find whether a count or an array sets the number of outputs.
A method of a UGen with outputs that takes `mul` or `add` must apply
`madd` once: with `mul` 0.5 and `add` 0.25, it makes the UGens that one
`madd` of its result makes.
Then, on each argument that expands, an array of two elements must make
two instances, instance i with element i. An array inside an array must
make three. Last, the array argument takes `[[e0, e1], e2]`, then
`[e2, [e0, e1]]`. The instances that they make give its `ArrayExpansion`.
If one instance keeps the array among its inputs, the method has none.
581 of the 635 methods are addable with a wiring, and 67 have a body. Most
of the others do more than `UGen:new1` does:

- The pseudo-UGens without a body, such as `Dunique`, create
  no instance of their class.
- `VarLag` builds an `Env` with its default warp.
- `UnaryOpUGen` and `BinaryOpUGen` take the name of an operator. Use
  `unary_op` and `binary_op` instead.
- `MulAdd.new` folds, and orders its inputs. Use `madd` instead.

The script also takes a census of the classes whose rate methods do more
than `new1`. Three tables name each class outside `UGen` that answers `ar`
or `kr`, and each method that creates no instance of its class. The first
table names the classes with a body, and the second the classes left out,
with a reason. The third names the classes that are not pseudo-UGens. If a
class is in no table, the script fails. Three more tables name the methods
of the Lag family, the arguments whose zeros become silence, and the
arguments that go to audio rate together. Probes test each entry. If one
does not hold, the script fails. Two more tables map the class that owns
`optimizeGraph` and `fftSize` to `optimize` and `fft_size`. One more maps
the class that owns `convertEnv` to `convert_env`. If no table names the
owner, the script fails. Last, the script writes the specs of
`Spec.specs` and the class methods of `Env` that make an envelope. It also
writes the methods of `PV_ChainUGen` that take a function. It fails on a
spec that is not a `ControlSpec`, and on a warp that is not a symbol. It
also fails on a class method of `Env`, or an instance method of
`PV_ChainUGen`, that no table names. So it does on an instance method of
`UGen`, and on a `signalRange` that is neither `\bipolar` nor
`\unipolar`. For each method of `UGen` that builds a graph, it writes the
classes whose methods a Float and an `Array` answer. It also writes the
subclasses that have a method of their own.

The script also reads the plugins at the tag. Each unit inside one of the
two macros that set `kUnitDef_CantAliasInputsToOutputs` gets `cant_alias`.
If a plugin sets the flag in another way, or for a class outside the core,
the script fails.

The catalogue gives the page of each class in the help, and not its
summary: the help of SuperCollider is GPL-3, and the crate copies none of
it.

### Methods that `ugen` refuses

`tests/readme.rs` keeps this list equal to the catalogue.

- These pseudo-UGens have no body. `Dunique.new` builds its reader at each
  use as an input, and `InBus.ar` and `InBus.kr` read a `Bus` of sclang.
- `MulAdd.new` folds and orders its inputs. Use `madd` instead.
- An input comes from no argument, or the inputs follow a count:
  `DecodeB2.ar`, `DecodeB2.kr` and `Dwrand.new`.
- So do `MaxLocalBufs.new`, `SendPeakRMS.ar`, `SendPeakRMS.kr` and
  `SetBuf.new`.
- They need arguments that the probe does not make, such as a `Ref`, a
  `Rect` or an operator: `BinaryOpUGen.new`, `InRect.ar` and `InRect.kr`.
- So do `LagControl.ir`, `OffsetOut.kr`, `OutputProxy.new` and
  `UnaryOpUGen.new`. Some of these do not exist.

## sc3-plugins

With the feature `sc3-plugins`, the module `sc3_plugins` holds the 491
UGen classes of sc3-plugins 3.14.0. Many installations of SuperCollider
add these plugins to the core. `ugen` adds them as it adds the classes of
`catalog`. `sc3_plugins::lookup` gives a class by name, and
`sc3_plugins::all` gives all of them, sorted by name.
`sc3_plugins::pseudo_lookup` and `sc3_plugins::pseudo_all` give the
classes outside `UGen` that the crate builds, such as `JPverb` and `BLPF`.
The classes have the types of `catalog`, and no name is in both
catalogues.

```rust
# #[cfg(feature = "sc3-plugins")]
# {
use scsynth_synthdef::{SynthDefBuilder, UGenInput};

// SynthDef(\verb, { Out.ar(0, JPverb.ar(SinOsc.ar([220, 330], 0, 0.1), 3)) })
let mut builder = SynthDefBuilder::new("verb", vec![])?;
let freqs = UGenInput::Array(vec![220.0.into(), 330.0.into()]);
let args = [("freq", freqs.into()), ("mul", 0.1.into())];
let pair = builder.ugen("SinOsc", "ar", &args)?;
let wet = builder.ugen("JPverb", "ar", &[("in", pair.into()), ("t60", 3.0.into())])?;
builder.ugen("Out", "ar", &[("bus", 0.0.into()), ("channelsArray", wet.into())])?;
let def = builder.finish()?;
assert!(def.children.iter().any(|ugen| ugen.name == "JPverbRaw"));
assert_eq!(def.wire_buffers()?, 4);
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

`JPverb` gives the two channels to `JPverbRaw`. That unit cannot alias, so
it takes the buffers of its two outputs before it frees those of its two
inputs: the definition needs four wire buffers.

- The reference is the tag `Version-3.14.0` of sc3-plugins. The installed
  folder `SC3plugins` of the Extensions holds its class files.
  `Version-3.14.1` changes `VBAPSpeakerArray` alone, which builds no UGen,
  so the catalogue serves both.
- 635 of the 679 rate methods are addable with a wiring, and four have a
  body. [Pseudo-UGens of sc3-plugins](#pseudo-ugens-of-sc3-plugins) lists
  the bodies.
- Most rules of `checkInputs` are those of the core. The 96 HOA classes
  ask for audio rate on their first n inputs at audio rate, for an n of 1
  to 121 by class. `BFGrainPanner` and its subclasses ask for it on input
  2.
- 176 units of sc3-plugins with a class set
  `kUnitDef_CantAliasInputsToOutputs`, such as `GreyholeRaw`, `JPverbRaw`
  and the HOA classes. `SynthDef::wire_buffers` counts them.
- The feature adds 258 KB to a release wasm32 build, and 28.5 KB after
  gzip. The core catalogue takes 102 KB, and 17 KB after gzip. The 96
  HOA classes hold two thirds of the arguments.

`generator/sc3_plugins/catalog.scd` measured each method with the probes of
`generator/catalog.scd`, in sclang 3.14.1 with the core class library and
the installed folder. It fails unless each class file of the folder is the
file of the tag. A table names the rule of `checkInputs` of each owner
outside HOA, and the script measures the n of each HOA class. Then it
gives the instances of every class inputs at audio and at control rate,
and their `checkInputs` must give the verdict of the crate. The script
also reads the C++ files of the plugins at the tag. A unit can set the
flag through the two macros of the core, or through a macro of its own
file. A FAUST plugin sets it in its call of `fDefineUnit`. If a unit sets the
flag in another way, or has no class outside ATK and three names of a
table, the script fails.

### Pseudo-UGens of sc3-plugins

`tests/readme.rs` keeps this list equal to the catalogue of sc3-plugins.
Each body follows the sclang code of its method, as the bodies of the core
do.

- `DEINDUGens`: `Greyhole.ar` and `JPverb.ar` call their raw class on the
  first and the last channel of `in`.
- `NHUGens`: `NHHall.ar` takes an array of two channels.
- `JoshUGens`: `HighShelf.ar`, `LowShelf.ar` and `Notch.ar` compute the
  coefficients of one `SOS`. `RBJHighShelf.ar`, `RBJLowShelf.ar` and
  `RBJNotch.ar` inherit these methods.
- `JoshUGens`: `ZLPF.ar` and `ZHPF.ar` add one `SOS` for each element of
  `damping`, and each `SOS` applies `mul` and `add`.
- `JoshUGens`: `BLPF.ar` and `BHPF.ar` call `ZLPF` or `ZHPF` with the
  dampings of their order. `LRLPF.ar` and `LRHPF.ar` call them twice, with
  half their order.
- `JoshUGens`: `Rotate.ar`, `Tilt.ar` and `Tumble.ar` turn two channels
  of B-format with `Rotate2`, and give the other two back as they come.
- `JoshUGens`: `BFFreeVerb.ar` and `BFGVerb.ar` put a reverb on each
  channel of `B2A`, mix each channel with two others, and give `A2B` of
  the mix. `BFGVerb` delays each channel by a `Rand` first.
- `JoshUGens`: `BtoUHJ.ar` and `UHJtoB.ar` mix the channels with their
  shifts by 90 degrees, as the two chains of six `FOS` of `Shift90.filt`
  give them.
- `BhobUGens`: `BhobLoShelf.ar` and `BhobHiShelf.ar` mix `in` with an
  allpass `FOS`. Their arguments have no defaults.

These bodies take the elements of an array as sclang destructures it, as
in `#a, b = Rotate2.ar(x, y, angle)`. When an argument is an array, the
expansion can give fewer elements than the destructuring takes. Then
sclang takes nil, and a later call throws. `ugen` refuses the argument
with `BuildError::WrongKind`.

The order of `BLPF` and `BHPF` is 2, 4, 6, 8 or 10, after `asInteger`
truncates it. The order of `LRLPF` and `LRHPF` is twice that. For another
order, sclang posts an error and gives a String, and a use of the result
throws. `ugen` refuses such an order with `BuildError::WrongKind`. It
refuses an output as an order too.

### Methods of sc3-plugins that `ugen` refuses

`tests/readme.rs` keeps this list equal to the catalogue of sc3-plugins.

- No plugin of sc3-plugins 3.14.0 defines the unit of `BlitB3D.ar`,
  `CQ_Diff.kr`, `Clipper32.ar` or `FFTSubbandFlux.kr`. Neither does one
  define that of `RMAFoodChainL.ar`, `RegaliaMitraEQ.ar` or
  `RosslerResL.ar`. sclang writes them, and scsynth refuses them with
  `UGen '<name>' not installed.`.
- `StkFlute.kr` applies `madd` twice.
- These methods create no instance of their class: `FFTCentroid.kr`,
  `OnsetsDS.kr`, `PV_DiffMags.new`, `PanX2D.ar` and `PanX2D.kr`. So does
  `SoftClipAmp.ar`.
- They put the size of an array before it: `FeatureSave.kr`,
  `FFTSubbandFlatness.kr`, `FFTSubbandPower.kr`, `MatchingPResynth.ar`
  and `MatchingPResynth.kr`.
- Their outputs come from a count in another way: `MatchingP.ar`,
  `MatchingP.kr` and `NearestN.kr`. `SOMRd.ar`, `SOMRd.kr` and
  `SpectralEntropy.kr` test numbers.
- They compute their inputs: `DbufTag.new`, `Dtag.new` and `Dfsm.new`
  from rules, `FM7.ar` from two matrices, and `LPCSynth.ar`, `LPCVals.ar`
  and `LPCVals.kr` from a clip of `pointer`.
- `Dneuromodule.ar` and `Dneuromodule.kr` put a `Duty` around
  `Dneuromodule.new`, which takes its arrays and its count in its own way.
- `TextVU.ar`, `TextVU.kr` and `TextVU.new` work as `Poll` does.
  `StkGlobals.ar` and `Getenv.new` write the codes of a text without its
  size first.

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

One limit depends on the server, so writing does not check it: the wire
buffers of a definition, against the `-w` of scsynth.
[Wire buffers](#wire-buffers) counts them.

### Wire buffers

`SynthDef::wire_buffers` gives the number of wire buffers that scsynth
3.14.1 gives a definition, as `DoBufferColoring` counts them
(`SC_GraphDef.cpp:873`). While the server runs, scsynth refuses a
definition that needs more than its `-w`, 64 by default: "exceeded number
of interconnect buffers." It then drops every definition of the file.
`GraphDef_Recv` prints the exception and goes on, so `/d_recv` still
answers `/done` (`SC_GraphDef.cpp:589`). scsynth does not report its `-w`,
so the caller compares the count with the `-w` that it gave the server.

```rust
use scsynth_synthdef::SynthDefBuilder;

// SynthDef(\chain, { Out.ar(0, LPF.ar(SinOsc.ar, 1000)) })
let mut builder = SynthDefBuilder::new("chain", vec![])?;
let sine = builder.ugen("SinOsc", "ar", &[])?;
let low = builder.ugen("LPF", "ar", &[("in", sine.into()), ("freq", 1000.0.into())])?;
builder.ugen("Out", "ar", &[("bus", 0.0.into()), ("channelsArray", low.into())])?;
assert_eq!(builder.finish()?.wire_buffers()?, 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

scsynth walks the UGens in file order:

- Each audio-rate output takes a buffer, with one reference for each input
  that reads it. Control-rate and scalar outputs take none.
- A UGen frees its references first, then takes buffers for its outputs.
  So an output can take a buffer that an input just freed, as `LPF` does
  above. The count is the most buffers in use at one time.
- A unit whose plugin sets `kUnitDef_CantAliasInputsToOutputs` takes its
  buffers first. 18 core units do, such as `PlayBuf`, `PanAz` and the
  `Grain` UGens. `UGenClass::cant_alias` holds the flag.
- A demand-rate UGen frees no buffer.
- An output that nothing reads keeps its buffer to the end.

A UGen whose class no catalogue of the crate holds gives
`WireBufferError::UnknownUGen`, because the count needs the flag of its
plugin. With the feature `sc3-plugins`, the classes of sc3-plugins count. A definition that breaks a limit gives `WireBufferError::Limit`.

### Traps

The crate does not rule these out, and neither does sclang:

- An array argument shorter than what the UGen reads. For example, a raw
  envelope for `EnvGen` with too few values makes scsynth read past the
  inputs of the unit.
- Two controls of one name, such as an argument and a `NamedControl`.
  scsynth sets the first of them at `/s_new` and `/n_set` (measured in
  NRT). Its table of parameters keeps the first name (`HashTable::Add`). A
  variant sets the last of them.
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
- A graph where `finishBuild` of sclang throws or writes a broken
  definition: `BuildError::FinishBuild` refuses it (see
  [`optimizeGraph` and the copies of PV chains](#optimizegraph-and-the-copies-of-pv-chains)).
- A rewrite that folds into a new UGen, as `0 + x.neg` becomes `x.neg`:
  `BuildError::FinishBuild` refuses it, where sclang rewrites it. Only
  `add` builds such an input.
- The checks of the inputs: when `ugen` adds a UGen, it checks the inputs.
  sclang checks after `optimizeGraph`, so a UGen that it removes escapes
  its check, as an unread `LPF.ar(SinOsc.kr)` does.
- An empty array where an argument expands, and an array that stays among
  the inputs of a UGen: the builder refuses them (see Multichannel
  expansion).
- An array of symbols: sclang expands a symbol argument too, as the command
  names of `SendReply` or the labels of `Poll`. `ArgValue` holds one text.
- The default label of `Poll` for a whole number names `Integer`. sclang
  names `Float` for a Float such as 440.0.
- A whole Float as `rotate` of `PingPong`, such as 2.0: `Array:rotate` of
  sclang takes only an Integer, and the crate takes it.
- Two numbers: `unary_op` and `binary_op` make a UGen of them, as
  `UnaryOpUGen.new` and `BinaryOpUGen.new` do. The operators of sclang
  compute them instead, as `2 * 3` gives 6. The crate cannot copy that.
  `2 < 3` is `true`, which no UGen takes. When sclang builds, `rand` draws.
  sclang gives nan for `-2 ** 0.5`, and the `pow` of scsynth keeps the
  sign.
- `mul` and `add` on a UGen without outputs, such as `AnalogOut`: sclang
  multiplies an output that the UGen does not have, and `optimizeGraph`
  removes the product. `ugen` refuses a value other than 1 and 0.
- An output in an array of buses of `SoundIn`: sclang builds a `+ 1` for
  its test of consecutive buses, and `optimizeGraph` removes it. `ugen`
  refuses the output.
- The warnings of `Dstutter` and `Mix.kr`: the crate does no I/O, and
  builds the same graph.
- An array of specs for `DynKlank`, `DynKlang`, `Klank` and `Klang`: sclang
  expands it, as an array of `Ref`s. `ugen` takes one spec.
- A `spread` or a `center` of `SplayAz` at scalar rate: the `blend` of its
  positions builds a `LinXFade2` outside the graph of sclang.
  `BuildError::UnsupportedRate` refuses it, and a demand rate too, where
  sclang throws.
- The `fill` methods of `Mix`, `Splay` and `SplayAz`, which take a
  function: build the array, then call `new`, `ar` or `kr`.
- Numbers are the Floats of sclang. sclang keeps its Integers apart, but
  gives the same values for the operators of the pseudo-UGens. `send`
  takes the methods of `Float` for every number. An Integer of sclang
  wraps with `hi` included: `3.wrap(0, 2)` gives 0, and `3.0.wrap(0, 2)`
  gives 1.0.
- The arguments of `send`: they come by name, and go to the method by
  position, with the defaults of the method of `UGen`. In sclang the
  method of a number or an array has defaults of its own, as `Float:clip`
  has none. An `Array` also drops the keywords of its methods.
- A nil argument with a default: `send` cannot pass it. Give `none` as the
  `clip` of the maps, which sclang takes as no clip too. Only `expexp` of a
  `DUGen` throws on a nil `clip`.
- An empty array as the receiver of `send`: sclang throws for most
  methods, and gives an empty array for some. `send` refuses it.
- A `Scale` as the `scale` of `degreeToKey` of a number: the crate takes
  an array.
- A spec as an array or a `ControlSpec`, such as `[0, 1, \lin, 0, 0.5]`:
  the `spec` of `NamedControl` takes a name of `catalog::specs`. Give the
  default of such a spec as the values. sclang keeps the spec only in the
  metadata, which the crate does not build.
- A `NamedControl` without a name: sclang names it `nil`, and `ugen`
  refuses it.
- Values and lags of `LagControl.kr` that differ in size: if nothing reads
  the result, sclang skips the call. `ugen` refuses it.
- `LagControl.kr` with no values: sclang adds nothing, and `ugen` refuses
  it.
- A release or loop node of an `Env` that is an output: the crate takes
  numbers, so `delay` needs no math.
- A name as the curve of a triplet of `Env.xyc`: use `Env.pairs` with a
  curve per segment.

## Left out

This is not in version 0.14, the next item of value for sc-app3:

1. The bodies of the other pseudo-UGens of sc3-plugins, such as
   `FreeVerb1` and `Vocoder`, and of the methods above that do more than
   `new1`.

Of sc3-plugins, the crate also leaves out ATK, whose classes come with the
quark `atk-sc3`. `CheapVerb` draws `rand` of the language at each build.
`BF`, `BFDecode`, `BFEncode` and `Reflections` compute with objects of the
language, such as `ZPoint`. `BhobTone` calls `HiShelf`, which no class
defines. `OnsetsDS` is deprecated, and gives a Function. `UGen:circleRamp`
has no body yet.

Also left out: the view of `SynthDesc`, with its inputs, outputs and
metadata, the specs included. So are the pseudo-UGens `AudioIn` and
`SplayZ`, deprecated since 3.10, and `InBus`, `XIn` and `XInFeedback` of
`JITLib`. `Dunique` builds a reader at each use as an input, and the crate
adds the UGens of a method at its call.

Of `Env`, the crate also leaves out the setters, the instance `circle` and
`blend`. The methods of the language, such as `at`, `plot` and `test`, are
out too. Of the instance methods of `UGen`, it leaves out those that give
objects of the language, such as `@` and `asComplex`. It also leaves out
`scope` and `belaScope`, which play.

## Tests

`cargo test --all-features` runs every test but the acceptance tests. It
needs no SuperCollider. `tests/readme.rs` checks the lists of methods that
`ugen` refuses, and the list of the pseudo-UGens of sc3-plugins.

The golden files in `tests/fixtures` are definitions that sclang 3.14.1
built and wrote. Each one reads, and writes back to its own bytes.
`generator/fixtures.scd` writes them again:

```sh
sclang -l "$PWD/generator/sclang_conf.yaml" "$PWD/generator/fixtures.scd"
```

- `generator/sclang_conf.yaml` compiles the core class library of
  `/Applications/SuperCollider.app` alone, without the Extensions folder and
  without quarks. For another installation, change its path.
- A graph that the crate refuses must break in sclang, or the script
  fails. The build or the write throws, or the definition is broken. A
  broken definition lacks an `Out`, holds a UGen twice, points a name past
  the parameters, or writes a variant without values.
- `tests/fixtures/arithmetic.json` holds operands and results of the
  operators that sclang computes on Floats, as the bits of each f64.
- `~/.bin/sclang` changes the directory, so the paths must be absolute.
- `generator/goldens.scd` holds the writers of both fixtures scripts.

The golden files of sc3-plugins are in `tests/fixtures/sc3_plugins`, and
`tests/sc3_plugins` tests them with the feature. `methods.scsyndef` holds
one definition for each addable method, which the crate builds from the
catalogue alone. `bodies.json` holds the cases of the bodies, and each
case builds in sclang and in the crate from that file. Each pseudo method
needs a case that builds. `generator/sc3_plugins/fixtures.scd` writes the
files again:

```sh
sclang -l "$PWD/generator/sc3_plugins/sclang_conf.yaml" \
  "$PWD/generator/sc3_plugins/fixtures.scd"
```

`generator/sc3_plugins/sclang_conf.yaml` compiles the core class library
and the folder `SC3plugins` of the Extensions, without the other
extensions and without quarks.

`generator/catalog.json` holds the catalogue and the tables of the
operators. The operators come from `Opcodes.h` and `initSpecialSelectors`
in the sources of SuperCollider, at the tag `Version-3.14.1`, and sclang
must agree with each special index. `generator/catalog.scd` writes the
JSON again, in two seconds:

```sh
SC_SOURCE=~/Projects/supercollider \
sclang -l "$PWD/generator/sclang_conf.yaml" "$PWD/generator/catalog.scd"
```

`generator/sc3_plugins/catalog.scd` writes
`generator/sc3_plugins/catalog.json`, the catalogue of sc3-plugins. It
reads the sources of sc3-plugins at the tag `Version-3.14.0`:

```sh
SC3_PLUGINS_SOURCE=~/Projects/sc3-plugins \
sclang -l "$PWD/generator/sc3_plugins/sclang_conf.yaml" \
  "$PWD/generator/sc3_plugins/catalog.scd"
```

Both scripts load `generator/probes.scd`, which measures the classes.
`tests/render.rs` renders the JSON into `src/operator/generated.rs`,
`src/catalog/generated.rs` and `src/sc3_plugins/generated.rs`. If a
committed file differs, the test fails. With `SCSYNTH_SYNTHDEF_BLESS=1`
set, it writes the file instead.

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
- Each golden definition loads at a `-w` equal to its count of wire
  buffers, and scsynth refuses it at one less. With
  `SCSYNTH_SYNTHDEF_BLESS=1` set, the test then writes
  `tests/fixtures/wire_buffers.json`, and `tests/wire_buffers.rs` tests the
  crate against it.
- A file with one definition over `-w` loads none of its definitions. Only
  this test starts synths, of definitions without an `AudioControl`. In NRT
  mode, scsynth 3.14.1 crashes on `/s_new` of a definition with an
  `AudioControl`.

`tests/sc3_plugins` has acceptance tests of its own, with the plugins of
sc3-plugins. They are ignored by default too:

```sh
cargo test --features sc3-plugins --test sc3_plugins -- --ignored
```

- `SCSYNTH_SC3_PLUGINS` names the folder of these plugins, by default the
  folder `SC3plugins` of the Extensions. `-U` takes it after the plugins of
  the bundle.
- scsynth must load every golden definition of sc3-plugins, at the most
  wire buffers that one of them needs. It must refuse the 7 classes
  without a unit with `UGen '<name>' not installed.`.
- Each golden definition loads at a `-w` equal to its count of wire
  buffers, and scsynth refuses it at one less. With
  `SCSYNTH_SYNTHDEF_BLESS=1` set, the test writes
  `tests/fixtures/sc3_plugins/wire_buffers.json`.

## Features

- `sc3-plugins`: the module `sc3_plugins`. `ugen`, the bodies and
  `SynthDef::wire_buffers` take the classes of sc3-plugins (see
  [sc3-plugins](#sc3-plugins)).
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
