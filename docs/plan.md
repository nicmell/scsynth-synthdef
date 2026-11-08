# The plan of scsynth-synthdef

This document is the plan of version 0.2. A session starts from it and from
the README, in this folder. The plan owns its decisions and facts until the
code and the tests own them. Then the facts move to the tests, the README
and the CHANGELOG, and this file keeps only what is still open.

## What we build

Version 0.2 adds the multichannel expansion of sclang to `ugen`, `unary_op`
and `binary_op`. It also adds a `madd` that does what `MulAdd.new` does, and
the folds of `BinaryOpUGen:new1`. sc-app3 needs it for `<sc-synthdef>`. Its
proposal is the artifact https://claude.ai/artifact/2jDyZdJ783ynSRsBiabrxP.

- The crate stays a library for any Rust user. Nothing of sc-app3 enters it,
  and it assumes no caller that folds constants or checks inputs first.
- A user who reads only the README and docs.rs gets the graphs of sclang, or
  a refusal that the docs explain.
- The crate keeps its rules: no I/O, serde as its only optional dependency,
  the MSRV 1.85, the `wasm32-unknown-unknown` build, its lints, and docs on
  every public item.

## Ground rules

- This is a clean implementation from the SuperCollider sources, as for 0.1.
  Do not consult earlier compilers: neither
  `sc-app2/packages/synthdef-compiler` nor
  `sc-crates/crates/scsynthdef-compiler`.
- No language enters the crate: no parse, no expression, and no evaluation
  of what a user writes. In sc-app3, sc-expression owns them.
- Where sclang writes a definition that scsynth refuses, the crate refuses
  the input.
- Never touch the scsynth on UDP 57110.

## Decisions

The owner took these on 2026-11-08, after the review of the proposal
against the sources.

1. The expansion follows `multiNewList`. The arguments that expand are the
   arguments that a method gives `multiNew`: the arguments of the wire
   (`InputSlot::Arg`) and the count of the outputs (`Outputs::Arg`). `mul`
   and `add` go to `madd`. An `Array` argument flattens one level into the
   inputs, as in 0.1. A symbol argument takes one text.
2. Counts expand, as in sclang (fact 6). The proposal said that they never
   expand.
3. `ugen` applies `madd` once, to the whole result of the expansion
   (fact 8). The proposal applied it to each instance.
4. An empty array where an input expands is refused at every depth, in
   `ugen`, `unary_op`, `binary_op` and `madd` (facts 16 and 17). A nested
   array in an `Array` argument is refused (fact 19). Each refusal has its
   own `BuildError` variant.
5. `binary_op` and `unary_op` are `BinaryOpUGen.new` and `UnaryOpUGen.new`.
   Two numbers make a UGen, as they do in sclang (facts 13 and 14). Only
   `madd` and the folds compute numbers: `neg`, `+`, `-` and `*`.
6. The generator measures the expansion of each addable method. A method
   that does not expand a flat array as `multiNewList` does is not addable,
   with the reason. A method that expands one level only is recorded, and
   `ugen` refuses a nested array there (fact 18).
7. A call that fails adds no UGen.
8. `add` stays at the level of `UGen:new1`, and refuses an array.

## Sources of truth

The SuperCollider source at the tag `Version-3.14.1`, in
`~/Projects/supercollider`, and sclang and scsynth 3.14.1:

- `SCClassLibrary/Common/Audio/UGen.sc`: `multiNew`, `multiNewList`,
  `madd`, `composeBinaryOp`, `reverseComposeBinaryOp`, `numInputs` and
  `writeDef`.
- `SCClassLibrary/Common/Audio/BasicOpsUGen.sc`: `UnaryOpUGen.new`,
  `BinaryOpUGen.new` and `new1`, and `MulAdd.new`, `new1` and
  `canBeMulAdd`.
- `SCClassLibrary/Common/Collections/Array.sc` (`madd`, `isValidUGenInput`),
  `SequenceableCollection.sc` (`performBinaryOp`,
  `performBinaryOpOnSeqColl`, `rate`) and `Collection.sc`
  (`writeInputSpec`).
- `SCClassLibrary/Common/Math/Number.sc` and `Core/AbstractFunction.sc`:
  the operators of numbers and of UGens on arrays.
- `SCClassLibrary/Common/Audio/InOut.sc` (`Out`, `In`), `Trig.sc`
  (`SendReply`), `EnvGen.sc` (`convertEnv`) and `SynthDef.sc`
  (`finishBuild`).
- `server/plugins/BinaryOpUGens.cpp`: the operators of scsynth.

sclang runs with `-l generator/sclang_conf.yaml`, the core class library
alone. If the script ends with `0.exit;`, a plain redirect of its output to
a file keeps all of it. scsynth runs from the bundle with `-U`, in NRT mode,
as `tests/scsynth.rs` runs it.

## Facts established on 2026-11-08

"Measured" means a run of sclang 3.14.1, or of scsynth 3.14.1 where the fact
says so.

### The expansion

1. `multiNewList` expands an argument whose class is `Array`, and no other.
   The size is that of the longest array. Instance i takes element i modulo
   the size of each array, and every other argument as it is. An element
   that is an array expands again, depth first. The instances come in the
   order of i. If no argument is an array, or if each array is empty, it
   calls `new1` with the arguments as they are. (`UGen.sc:23-39`)
2. `finishBuild` collects the constants before the topological sort, so the
   order of creation shows in the table of constants.
   (`SynthDef.sc:279-287`)
3. Measured: `SynthDef(\pair, { |freqs = #[220, 331], amp = 0.4|
   Out.ar(0, SinOsc.ar(freqs) * amp) })` writes `Control`, `SinOsc`, `*`,
   `SinOsc`, `*` and `Out`, in 280 bytes.
4. Measured: `Out.ar(0, SinOsc.ar([440, 442, 444]) * [0.1, 0.2])` writes
   `SinOsc`, `*`, `SinOsc`, `*`, `SinOsc`, `*` and `Out`, with the constants
   [440, 0, 442, 444, 0.1, 0.2]. The three `SinOsc` come before the three
   `*`.
5. Each count argument of an addable method goes to `multiNew`: 21
   arguments of 17 classes, as `In.ar` calls
   `multiNew('audio', numChannels, bus)`. (sources)
6. Measured: `In.ar(0, [1, 2])` makes two `In`, of 1 and 2 channels.
7. Measured: `SendReply.kr(Impulse.kr(1), ['/a', '/b'], [1, 2])` makes two
   `SendReply`. A symbol argument expands too. `ArgValue::Text` holds one
   text, so the builder cannot express it.

### madd

8. Each rate method with `mul` and `add` ends in `.madd(mul, add)`, on the
   result of `multiNew`. A scan of `Common/Audio` finds no other form, but
   `MulAdd.new`. `UGen:madd` and `Array:madd` call `MulAdd(this, mul, add)`
   (`UGen.sc:51`, `Array.sc:281`), and `MulAdd.new` expands over `in`,
   `mul` and `add` together. Measured: `SinOsc.ar([440, 442], 0, [0.1, 0.2,
   0.3])` makes 2 `SinOsc` and 3 `*`: s0 * 0.1, s1 * 0.2 and s0 * 0.3.
   `SinOsc.ar([440, 442], 0, 0.5)` has the constants [440, 0, 442, 0.5].
9. `MulAdd.new1` (`BasicOpsUGen.sc:418-443`) takes the first case that
   holds:
   - If `mul` is 0, the result is `add`.
   - If `mul` is 1 and `add` is 0, the result is `in`.
   - If `mul` is -1 and `add` is 0, the result is `in.neg`.
   - If `add` is 0, the result is `in * mul`.
   - If `mul` is -1, the result is `add - in`.
   - If `mul` is 1, the result is `in + add`.
   - If `canBeMulAdd(in, mul, add)` holds, the result is a `MulAdd` of `in`,
     `mul` and `add`.
   - If `canBeMulAdd(mul, in, add)` holds, the result is a `MulAdd` of `mul`,
     `in` and `add`.
   - Otherwise, the result is `(in * mul) + add`.

   `canBeMulAdd` holds in two cases: `in` runs at audio rate, or `in` runs at
   control rate with `mul` and `add` at control or scalar rate. The operators
   compute two numbers.
10. Measured: `SinOsc.ar(440, 0, 0.1)` gives a `SinOsc` and a `*`, and
    `SinOsc.ar(440, 0, 0.1, 0.5)` a `SinOsc` and a `MulAdd`.
    `MulAdd(SinOsc.kr(1), SinOsc.ar(2), 0.5)` writes a `MulAdd` of the
    `SinOsc.ar`, the `SinOsc.kr` and 0.5, at audio rate.
    `MulAdd(SinOsc.kr(1), 0.5, SinOsc.ar(2))` writes a `*` at control rate
    and a `+` at audio rate. `MulAdd(2, 3, SinOsc.ar(2))` writes
    `6 + SinOsc`.
11. The rate of a `MulAdd` is `inputs.rate`: the smallest rate by the name
    of its symbol, so audio, control, demand, scalar
    (`SequenceableCollection.sc:1207`). Measured:
    `MulAdd(SinOsc.ar(1), Dseq([1, 2], inf), 0.5)` runs at audio rate, where
    a `BinaryOpUGen` of the same inputs runs at demand rate.

### The folds and the operators

12. `BinaryOpUGen.new1` folds these cases, in this order
    (`BasicOpsUGen.sc:73-97`):
    - `0 * b` and `a * 0` give 0.0.
    - `1 * b` gives b, and `-1 * b` gives `b.neg`.
    - `a * 1` gives a, and `a * -1` gives `a.neg`.
    - `0 + b` gives b, and `a + 0` gives a.
    - `0 - b` gives `b.neg`, and `a - 0` gives a.
    - `a / 1` gives a, and `a / -1` gives `a.neg`.

    `==` compares numbers, an Integer with a Float too. A UGen equals only
    itself (`Object.sc:181`), so it never folds. `neg` of a number is a
    number, and of a UGen a `UnaryOpUGen`. Measured: `SinOsc.ar(440) * 1`
    and `* 1.0` make no operator, `0 - SinOsc.ar(440)` makes a `neg`, and
    `BinaryOpUGen('-', 0, 3)` is -3.
13. `UnaryOpUGen.new` has no fold. Measured: `UnaryOpUGen('neg', 3)` writes
    a `UnaryOpUGen` at scalar rate, and `BinaryOpUGen('*', 2, 3)` writes a
    `BinaryOpUGen` of the constants 2 and 3.
14. The crate cannot copy the operators of sclang on numbers. `2 < 3` is
    `true`, which no UGen takes (measured: "binary operator '*' failed").
    `rand`, `rrand` and `coin` draw at build time. `UGenInput::Number` is an
    f64, without the Integer of sclang. sclang gives nan for `-2 ** 0.5`
    (measured), and the `pow` of scsynth keeps the sign
    (`BinaryOpUGens.cpp:1261`).
15. A UGen with an array goes through `BinaryOpUGen.new`, so through
    `multiNewList` (`UGen.sc:437-446`, `AbstractFunction.sc:28`,
    `Array.sc:217`). Two arrays, or an array and a number, go through
    `performBinaryOpOnSeqColl` (`SequenceableCollection.sc:1106-1118`,
    `Number.sc:13`). Both give the same shapes and the same order for arrays
    that are not empty.

### Empty and nested arrays

16. `numInputs` counts an empty array as one input (`UGen.sc:489`), and
    `Collection:writeInputSpec` writes nothing for it (`Collection.sc:723`).
    Measured: `SynthDef(\e, { Out.ar(0, SinOsc.ar([])) })` writes 92 bytes.
    scsynth answers "exception in GraphDef_Recv: UGen '' not installed.",
    and the reader of the crate "at byte 75: the count -256 is negative".
17. Measured: `SinOsc.ar([440, 442], [])` fails with "SinOsc arg: 'phase'
    has bad input: nil". `SinOsc.ar([[], 440])` and `SinOsc.ar * []` write
    the same broken input. `[] + 1` and `[] + []` are `[]`, `[1, 2] + []`
    fails, `BinaryOpUGen('*', [], 1)` is `[]` through its fold, and
    `UnaryOpUGen('neg', [])` makes a `UnaryOpUGen` without inputs.
18. `SendReply` expands with `flop`, then calls `new1`: one level, without
    recursion (`Trig.sc:31-43`). An array inside an argument becomes one
    input of `new1`.
19. `Out` joins its channels to the arguments of `multiNewList`
    (`InOut.sc:256-259`). Measured: `Out.ar(0, [[a, b], [c, d]])` makes
    `Out(0, a, c)` and `Out(0, b, d)`. `SendReply` with the values
    [[1, 2], [3, 4]] makes two `SendReply`, of [1, 2] and of [3, 4]. `EnvGen`
    with two envelopes makes two `EnvGen` (`EnvGen.sc:72-86`). 38 of the 61
    `Array` arguments belong to addable methods.

### The Bela outputs

20. `AnalogOut` and `DigitalOut` have no output, and apply `madd`.
    Measured: `AnalogOut.ar(0, SinOsc.ar(1), 2)` builds a `*` that reads the
    `AnalogOut`. `optimizeGraph` removes the `*`, since nothing reads it.

## Version 0.2

### The calls

- A call that fails adds no UGen: the builder truncates the UGens that the
  call added.
- `ugen` binds the arguments as in 0.1. It expands over the arguments of the
  wire and the count. Each instance goes through the wiring, the rule of
  `checkInputs` and `add`, as in 0.1. If the method applies `madd`, `ugen`
  then calls `madd` once on the whole result.
- A method without outputs that applies `madd` takes `mul` 1 and `add` 0
  only, and skips `madd` (fact 20). `BuildError::MaddNotSupported` refuses
  other values.
- `unary_op` expands over its input, and adds a `UnaryOpUGen` for each
  element, a number too.
- `binary_op` expands over its two inputs, and applies the folds of fact 12
  to each pair before it adds a `BinaryOpUGen`.
- `madd(input, mul, add)` expands over its three inputs, and applies
  `MulAdd.new1` (fact 9) to each triple. Its operators compute two numbers
  in f64, and go to the leaf of `binary_op` otherwise.

### The expansion

One helper serves the four calls. It takes the values that expand, with
their names:

- An empty array, at any depth, is `BuildError::EmptyInput`.
- At depth 1 and below, an array in a method that expands one level is
  `BuildError::NestedArray`.
- The size is that of the longest array. Instance i takes element i modulo
  the size of each array. An element that is an array expands again. The
  result is one value, or an array of the instances in order.

An element of an `Array` argument that is itself an array is
`BuildError::NestedArray`.

### The catalogue

- `Wiring` gains `nested_expansion`: whether an array inside an array
  expands the method again.
- `generator/catalog.scd` measures it. For each argument that expands, an
  array of two elements must make two instances and nothing else, instance
  i with element i. The counts take [2, 3]. Otherwise the method is not
  addable. Then an array that holds an array must make three instances, in
  order. Otherwise `nested_expansion` is false.

### The errors

- `EmptyInput { ugen, argument }` and `NestedArray { ugen, argument }`
  name the class and the argument. The operators name their class
  (`UnaryOpUGen`, `BinaryOpUGen`, `MulAdd`) and the arguments `a`, `b`,
  `in`, `mul` and `add`.
- `MultichannelExpansion` stays, for `add` only.
- `MaddNotSupported` refuses `mul` and `add` on a method without outputs.

## Differences from sclang that remain

The README lists them, with those of 0.1.

- An empty array where an input expands: the builder refuses it (facts 16
  and 17).
- A nested array in an `Array` argument: sclang expands the UGen, in a way
  that depends on the class (fact 19). The builder refuses it.
- An array of symbols: sclang expands the UGen (fact 7). `ArgValue` holds
  one text.
- A nested array in a method that expands one level, as `SendReply`: sclang
  writes the inner array as one input (fact 18). The builder refuses it.
- `mul` and `add` on a method without outputs: sclang drops them through
  `optimizeGraph` (fact 20). The builder refuses them.
- Two numbers: `unary_op` and `binary_op` make a UGen, as `UnaryOpUGen.new`
  and `BinaryOpUGen.new` do. The operators of sclang compute them (facts 13
  and 14).
- A product with 0 reaches `Out.ar` as the number 0, which the rule of
  `AbstractOut` refuses. sclang puts a `DC` in its place.

## Tests

- Golden files that sclang 3.14.1 writes with `generator/fixtures.scd`. Each
  one is a graph that `optimizeGraph` leaves alone:
  - `pair` (fact 3) and `wrap` (fact 4)
  - `voices`: `Pan2` of two voices, summed
  - `nested`: an `LPF` over the outputs of two `Pan2`, summed
  - `in_buses`: `In.ar([4, 6], [1, 2]).flat`
  - `mul`, `mul_add`, `mul_array`, `mul_expanded`, `mul_swap`, `mul_ops`
    and `madd_numbers` (facts 8 and 10)
  - `folds`: `* 1`, `0 -`, `mul` -1, `/ -1`, `+ 0` and `* -1`
- `tests/common` builds each golden graph with `ugen`, `binary_op` and
  `madd`. `tests/parity.rs` compares the bytes, and `tests/scsynth.rs` loads
  them in NRT mode.
- Unit tests:
  - each refusal, at the top and below it
  - a failed call that adds no UGen
  - each fold, with numbers and with outputs
  - two numbers that make a UGen, and `* 0` that gives 0.0
  - each case of `MulAdd.new1`, and the rate of a `MulAdd` with a demand
    input
  - `AnalogOut` with `mul` 2
- A test of the catalogue: `SendReply` expands one level, and `SinOsc` all
  the way.

## Steps

One commit for each step, on the branch `feat/expansion`. Each commit passes
the commands of CLAUDE.md.

1. docs: the plan of version 0.2.
2. test: the golden files of 0.2, written by sclang 3.14.1.
3. feat: the expansion of each rate method, measured in sclang.
4. feat: multichannel expansion in `ugen`, `unary_op` and `binary_op`.
5. feat: `madd`, and the folds of `BinaryOpUGen:new1`.
6. docs: the release of version 0.2. The README, the CHANGELOG, the version
   0.2.0, and this file trimmed to what is still open.

Then the branch reaches `main` through a pull request, the tag `v0.2.0`
marks the merge, and the proposal of sc-app3 records what changed.

## After version 0.2

These stay open, in the order of their value for sc-app3:

1. nested arrays in `Array` arguments, as each class expands them
2. `optimizeGraph`, and `addCopiesIfNeeded` for PV chains
3. what rate methods do besides `new1`: the Lag family,
   `asAudioRateInput`, `LocalBuf`, `Poll` and the zeros of `Out`
4. `NamedControl`, `Control.names`, `SynthDef.wrap`, `prependArgs` and the
   specs of the metadata
5. `Env`, and the pseudo-UGens such as `Mix`, `Splay` and `SoundIn`
6. the rules of `checkInputs` that classes define for themselves
7. sc3-plugins
