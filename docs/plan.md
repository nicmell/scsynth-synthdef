# The plan of scsynth-synthdef

This document is the plan of version 0.3. A session starts from it and from
the README, in this folder. The plan owns its decisions and facts until the
code and the tests own them. Then the facts move to the tests, the README
and the CHANGELOG, and this file keeps only what is still open.

## What we build

Version 0.3 adds the pseudo-UGens of the core class library of
SuperCollider 3.14.1: the classes whose rate methods build a graph of other
UGens, such as `Mix`, `Splay` and `SoundIn`. scsynth has no unit of these
names. `ugen` adds a pseudo-UGen by its class name, as it adds a UGen of the
catalogue. It builds what sclang builds, in the order of sclang. The
proposal is the artifact
https://claude.ai/artifact/6UCtwrR3NfwTgvkLmeVgUu.

- The crate stays a library for any Rust user. Nothing of sc-app3 enters it.
- A user who reads only the README and docs.rs gets the graphs of sclang, or
  a refusal that the docs explain.
- The crate keeps its rules: no I/O, serde as its only optional dependency,
  the MSRV 1.85, the `wasm32-unknown-unknown` build, its lints, and docs on
  every public item.

## Ground rules

- This is a clean implementation from the SuperCollider sources, as for 0.1
  and 0.2. Do not consult earlier compilers: neither
  `sc-app2/packages/synthdef-compiler` nor
  `sc-crates/crates/scsynthdef-compiler`.
- No language enters the crate: no parse, no expression, and no evaluation
  of what a user writes. The arithmetic that a pseudo-UGen does on its own
  arguments is part of its build, and stays private.
- Where sclang writes a definition that scsynth refuses, the crate refuses
  the input.
- A crate item lands with its first caller, so that no commit warns about
  dead code.
- Never touch the scsynth on UDP 57110.

## Decisions

The owner took these on 2026-11-13, from the recommendations of the
proposal.

1. Waves 1 and 2 come now, before items 2 to 4 of "After version 0.2". Wave 3
   waits for the parts of item 3 that it needs, except those of decision 3.
2. `ugen` adds a pseudo-UGen by class name, with the arguments by name and
   their defaults. There is no typed function for each pseudo-UGen. The
   catalogue holds the classes outside `UGen` in a table of their own,
   `PseudoClass`, beside the 419 `UGenClass`.
3. The `asAudioRateInput` of the 19 methods that call it at audio rate (the
   18 delays and `DelTapWr.ar`), and the `* level` of `LinXFade2`, come with
   this work, before wave 3.
4. A graph that `optimizeGraph` rewrites gets a second golden file, built by
   `finishBuild` without `optimizeGraph`, beside the file of the full build.
   The crate is tested against the second.
5. `DynKlank` and `DynKlang` take one spec: an array of up to three arrays.
   sclang expands an array of several specs, as an array of `Ref`s. That
   stays out.
6. The arithmetic of the language is private to the crate, with the
   operators that the bodies use. `binary_op` and `unary_op` keep the rule of
   0.2: two numbers make a UGen.
7. Out of this work: `AudioIn`, `SplayZ`, `InBus`, `XIn`, `XInFeedback`,
   `NamedControl`, and the `fill` methods that take a function.

The planning of 2026-11-13 corrected the proposal in three places:

- If `spread` or `center` is a UGen, the `resamp1` of `SplayAz` builds a
  `LinXFade2` or an `XFade2` (fact 10). So `SplayAz` moves from wave 2 to
  step 7, after `LinXFade2`.
- The arithmetic gets no step of its own. Each part lands with the first
  body that calls it.
- `PingPong` and `Dunique` are in the tables of the census from the first
  step, with `EnvGate`.

## Sources of truth

The SuperCollider source at the tag `Version-3.14.1`, in
`~/Projects/supercollider`, and sclang and scsynth 3.14.1:

- `SCClassLibrary/Common/Audio`, for most pseudo-UGens: `Mix.sc`,
  `Splay.sc`, `Osc.sc`, `Line.sc`, `SoundIn.sc`, `FSinOsc.sc` and
  `BEQSuite.sc`.
- The same folder, for the others: `Filter.sc`, `BufIO.sc`, `Demand.sc`,
  `FFTUnpacking.sc`, `Compander.sc` and `Hilbert.sc`.
- The same folder, for the build: `BasicOpsUGen.sc`, `Delays.sc`, `Pan.sc`,
  `UGen.sc` and `SynthDef.sc`.
- `SCClassLibrary/backwards_compatibility/PMOsc.sc`: `PMOsc` and `MultiTap`.
- `SCClassLibrary/Common/Collections`: `Collection.sc` (`sum`),
  `SequenceableCollection.sc` (`rate`, `clump`, `resamp1`, `sort`) and
  `Array.sc`.
- `SCClassLibrary/Common/Core/Object.sc` and `AbstractFunction.sc`, and
  `Common/Math/SimpleNumber.sc`: `dup`, `blend`, `blendAt`,
  `asAudioRateInput`, `moddif` and `series`.
- `lang/LangSource/PyrMathOps.cpp` and `lang/LangPrimSource/PyrMathPrim.cpp`:
  the operators of sclang on numbers. They call the double functions of
  `include/plugin_interface/SC_InlineBinaryOp.h`, `SC_InlineUnaryOp.h` and
  `include/common/SC_BoundsMacros.h`.

sclang runs with `-l generator/sclang_conf.yaml`, the core class library
alone. A probe wraps its body in `try`, with `0.exit` at its end and `1.exit`
in its handler: a probe that throws without an exit leaves sclang at its
prompt. A plain redirect of the output to a file keeps all of it. scsynth
runs from the bundle with `-U`, in NRT mode, as `tests/scsynth.rs` runs it.

## Facts established on 2026-11-13

"Measured" means a run of sclang 3.14.1.

### The census

1. The core class library has 34 pseudo-UGens: 22 classes outside `UGen`,
   and 12 subclasses of `UGen` that create no instance of their class.
   - Outside `UGen`, with a body in this work: `BHiPass4`, `BLowPass4`,
     `LinLin`, `LinSelectX`, `Mix`, `MultiTap`, `NumChannels` and `PMOsc`.
   - Also with a body: `SelectX`, `SelectXFocus`, `Silent`, `SoundIn`,
     `TChoose` and `TWChoose`, with `LevelComp` as a private helper.
   - Outside `UGen`, without a body: `EnvGate`, `PingPong`, `NamedControl`,
     `SplayZ`, `AudioIn`, `XIn` and `XInFeedback`.
   - Subclasses of `UGen`: `Changed`, `CompanderD`, `Dstutter`, `Dunique`,
     `DynKlang`, `DynKlank`, `HilbertFIR`, `InBus`, `Splay`, `SplayAz`,
     `Tap` and `UnpackFFT`.

   Measured: a probe lists the classes outside `UGen` with a class method of
   their own of these names: `ar`, `kr`, `ir`, `new`, `new1`, `fill`,
   `arFill` and `krFill`. In the audio classes, `backwards_compatibility`,
   `GraphBuilder.sc` and JITLib, it finds 21 of them. `LinSelectX` inherits
   from `SelectX`. The catalogue of 0.2 gives the 12.
2. The catalogue refuses some other methods as creating no instance. They
   are not pseudo-UGens. For a number or a lag of 0, the Lag family returns
   its input (`Filter.sc:94`). So do `Ramp`, `Slew` (`Filter.sc:283`),
   `VarLag` and `LagControl`. `MulAdd.new` folds. `Ndef` answers `ar`
   and `kr` for a proxy of JITLib.

### The build

3. sclang evaluates a receiver before its arguments, and the arguments from
   left to right. Each `new1` adds its UGen at once (`addToSynth`). So the
   UGens arrive in the order of the source, and the topological sort of the
   crate then gives the order of sclang. Measured: `SelectX.ar(which, [a,
   b, c])` adds `round`, `Select`, `trunc`, `+`, `Select`, `*`, `-`,
   `fold2` and `XFade2`, in this order.
4. Two numbers compute in the language. The operators of `SimpleNumber`
   call `doSpecialBinaryArithMsg` (`PyrMathOps.cpp:584`) and the double
   functions of `SC_InlineBinaryOp.h`: `sc_round` (268), `sc_roundUp` (274),
   `sc_trunc` (280), `sc_mod` (49), `sc_fold2` (555), and `sc_abs(a - b)`
   for `absdif`. `sqrt` is the `sqrt` of C (`PyrMathOps.cpp:443`), and
   `Float:clip` is `sc_clip` (`PyrMathPrim.cpp:594`,
   `SC_BoundsMacros.h:31`). Measured: `LinLin.kr(0.5, 0, 1, 10, 20)` is 15.0,
   `4 / 2` is the Float 2.0, and `Mix([1, 2])` is 3.
5. An operator on an array goes element by element, and the shorter of two
   arrays wraps (`SequenceableCollection.sc:1110`, fact 15 of the plan of
   0.2).
6. `Collection:sum` starts from 0 and adds each element
   (`Collection.sc:273`). `0 + x` folds to `x` for a UGen.
7. The rate of an array is the least rate by name: audio, control, demand,
   scalar (`SequenceableCollection.sc:1207`). A number is at scalar rate.
8. `dup` and `!` fill an array with copies (`Object.sc:162`), and
   `UGen:copy` gives the UGen itself (`UGen.sc:45`). So `Silent.ar(2)`
   repeats one `DC`, and `NumChannels` repeats one output.
9. `UGen:clip` adds a `Clip` at the rate of the UGen, and builds
   `max(lo, min(hi, x))` at demand rate (`UGen.sc:89`). `UGen:moddif` adds a
   `ModDif` at the rate of the UGen, demand too (`UGen.sc:282`).
   `SimpleNumber:moddif` computes `mod * 0.5 - absdif(absdif(a, b) % mod,
   mod * 0.5)` (`SimpleNumber.sc:547`). Measured: `SelectXFocus.ar` with
   `wrap` adds a `ModDif` for each input.
10. `resamp1` blends neighbours (`SequenceableCollection.sc:358`,
    `Object.sc:634-642`). If one side of `UGen:blend` runs at audio rate, it
    adds an `XFade2`. Otherwise it adds a `LinXFade2`, through
    `methodSelectorForRate` (`UGen.sc:122`, `UGen.sc:393`). It throws at
    demand rate. At scalar rate the selector is `new`, which is
    `Object:new`: the `LinXFade2` stays outside the graph. Measured:
    `SplayAz.ar(4, SinOsc.ar([1, 2, 3]), SinOsc.kr(0.1))` adds three
    `LinXFade2`.

### The expansion

11. `Splay` expands through `multiNewList` over `spread`, `level`,
    `center`, `levelComp` and each element of `inArray` (`Splay.sc:35-42`).
    `n` is at least 2 (`Splay.sc:27`). Measured:
    `Splay.ar([SinOsc.ar([1, 2]), SinOsc.ar([3, 4])])` makes two Splays of
    two inputs, and `Splay.ar(SinOsc.ar(1))` two `Pan2`.
12. `DynKlank` and `DynKlang` expand through `multiNew`, and a `Ref` does not
    expand (`FSinOsc.sc:70`, `FSinOsc.sc:92`). `Sum3` and `Sum4` expand
    through `multiNew` (`BasicOpsUGen.sc:458-497`).

### `Sum3`, `Sum4` and `Mix`

13. `Sum3:new1` and `Sum4:new1` fold an input that is 0. If `in2` is 0,
    `Sum3` gives `in0 + in1`. If `in1` is 0, it gives `in0 + in2`. If `in0`
    is 0, it gives `in1 + in2`. If an input of `Sum4` is 0, it gives a
    `Sum3` of the other three. Otherwise the UGen takes the rate of its
    inputs (fact 7), and sorts them with `a.rate < b.rate`. `sort` calls
    `mergeSort`, which runs `insertionSortRange` up to 9 elements
    (`SequenceableCollection.sc:1264`, `1372`, `1425`). That sort moves an
    input left past every input whose rate name is not less than its own.
    Measured: `Mix([kr1, ar2, kr3])` gives `Sum3(ar2, kr3, kr1)`, and
    `Mix([kr1, ar2, kr3, ar4])` gives `Sum4(ar4, ar2, kr3, kr1)`.
14. `Mix.new` clumps its array by 4, and mixes each clump with a `Sum4`, a
    `Sum3` or `sum`. It gives the `sum` of fewer than three mixes, a `Sum3`
    of three, and else mixes the mixes again (`Mix.sc:2`). Measured: 3
    inputs give a `Sum3`, 5 a `Sum4` and a `+`, 9 two `Sum4` and a `Sum3`.
    `Mix([1, 2, 3])` gives a `Sum3` at scalar rate with three constants, and
    `Mix([])` gives 0.
15. `Mix.ar` puts a `K2A` after a control-rate mix, and a `DC` after a
    number. `Mix.kr` puts an `A2K` before each audio-rate input, and a
    `DC.kr` after a number. Both throw at any other rate (`Mix.sc:34`,
    `Mix.sc:43`). Measured: the warning of `Mix.kr` builds no UGen.

### The bodies

16. `LinLin.ar` calls `MulAdd(in, scale, offset)`, and `LinLin.kr` writes
    `in * scale + offset` (`Line.sc:29`). Measured: `LinLin.kr` of a UGen is
    a graph that `optimizeGraph` rewrites.
17. `SoundIn.ar` reads from `NumOutputBuses.ir + bus`: one `In` of n channels
    for consecutive buses, else one `In` for each bus (`SoundIn.sc:1`).
    Measured: `[0, 1]` gives one `In` of two channels, and `[0, 2]` two `In`
    and one `+`. A UGen in the array makes the test of consecutive buses add
    a `+ 1`, which `optimizeGraph` removes.
18. `BLowPass4` and `BHiPass4` take `sqrt(rq)`, and chain two `SOS` with the
    coefficients of `BLowPass.sc` and `BHiPass.sc` (`BEQSuite.sc:8`, `28`,
    `172`, `181`). Measured: `BLowPass4.ar(in, 800, 0.5)` adds 20 UGens.
    One is an unused `SampleRate.ir`, which stays, because `SampleRate` is
    not pure.
19. The other bodies are in these places:
    - `PMOsc` and `MultiTap`: `backwards_compatibility/PMOsc.sc:4`, `17`.
    - `Changed`: `Filter.sc:227`. `Silent`: `Line.sc:112`. `Tap`:
      `BufIO.sc:136`.
    - `Dstutter`, which gives a `Ddup` and warns once: `Demand.sc:151`.
    - `SelectX`, `LinSelectX`, `SelectXFocus`, `TChoose` and `TWChoose`:
      `Osc.sc:256`, `276`, `282`, `317`, `329`.
    - `NumChannels`: `Mix.sc:76`. `LevelComp`, `Splay` and `SplayAz`:
      `Splay.sc:1`, `23`, `52`.
    - `DynKlank` and `DynKlang`: `FSinOsc.sc:68`, `90`. `UnpackFFT`:
      `FFTUnpacking.sc:7`.
    - `CompanderD`: `Compander.sc:23`. `HilbertFIR`: `Hilbert.sc:17`.
20. `SelectX.ar` takes `wrap` and hands it to `new1`, which has no such
    argument, so it changes nothing (`Osc.sc:257-265`).
21. `UnpackFFT` builds its bins with `(frombin..tobin)`, which is
    `prSimpleNumberSeries` (`PyrMathPrim.cpp:816`). Measured: `(0..3.0)` is
    `[0.0, 1.0, 2.0, 3.0]`, and `(2..0)` is `[2, 1, 0]`.

### `optimizeGraph`

22. Measured on 42 typical graphs: `optimizeGraph` rewrites `LinLin.kr` of a
    UGen, `DynKlank` of three partials, `DynKlang` with amplitudes,
    `MultiTap` and `PingPong`, and leaves the other 37. `DynKlank` with
    amplitudes is rewritten too.
23. A build that skips `optimizeGraph` writes bytes: `topologicalSort` runs
    `initTopoSort` itself (`SynthDef.sc:279`, `525`). Measured: `LinLin.kr`
    writes 203 bytes with a `*` and a `+`, and the full build 165 bytes with
    a `MulAdd`.

### The delays and `LinXFade2`

24. 19 methods pass their input through `asAudioRateInput` before
    `multiNew`: `DelayN`, `CombN`, `BufDelayN`, `BufCombN` and `DelTapWr`,
    with their subclasses (`Delays.sc:32`, `46`, `67`, `81`, `108`). A UGen
    that is not at audio rate gets a `K2A` (`AbstractFunction.sc:229`). A
    number gets a `DC`, and a number equal to 0 gets `Silent.ar`, which is
    `DC.ar(0)` (`SimpleNumber.sc:595`). An array converts element by element
    (`Array.sc:214`), before the expansion.
25. `LinXFade2.ar` and `LinXFade2.kr` multiply the result of `multiNew` by
    `level` (`Pan.sc:164`).

## Version 0.3

### The catalogue

- `generator/catalog.scd` gains three tables. `bodies` names the classes
  whose rate methods the crate builds with a body. `leftOut` names the
  classes of the census without a body, each with its reason. A class whose
  body comes in a later step has the reason of that step. `notPseudo` names
  the classes of fact 2, each with its reason.
- If no table names a class outside `UGen` that answers a class method `ar`
  or `kr`, the script fails. If no table names a class whose method creates
  no instance in its probe, the script fails. If an entry names no class,
  the script fails.
- A class of `bodies` outside `UGen` is described with its rate methods:
  `ar`, `kr`, `ir` and `new`, but not the `new` of `Object`. Each method
  gives the names, the defaults and the kinds of its arguments. No probe
  runs on a method of `bodies`.
- A new kind, `bool`, for `wrap` of `SelectXFocus` and `mixdown` of
  `NumChannels`. `levelComp` of `Splay` and `SplayAz` stays a signal with
  the default `true`.
- `generator/catalog.json` gains the key `pseudo_classes`, and the methods
  of `bodies` take `"addable": "pseudo"`. `tests/render.rs` renders them.
- The module `catalog` gains `PseudoClass`, `pseudo_lookup`, `pseudo_all`,
  `Addable::Pseudo` and `ArgKind::Bool`.
- Step 6 adds `Wiring::audio_input`, the position of the argument that goes
  through `asAudioRateInput`, and `Wiring::level`, whether the method
  multiplies its result by `level`. The probe gives that argument an
  audio-rate output, or the value 1.

### The entry and the bodies

- `ugen` finds the method among the UGen classes, then among the pseudo
  classes. For `Addable::Pseudo`, it binds the arguments by name, with their
  defaults, and leaves out an argument whose default is nil. Then it runs
  the body of the method. A call that fails adds no UGen.
- A body follows its sclang method line by line, in the order of fact 3. It
  adds the inner UGens with `ugen`, so the catalogue gives their wiring,
  their defaults, their checks and `madd`.
- `Splay`, `DynKlank`, `DynKlang`, `Sum3` and `Sum4` expand before their
  body, through the expansion of 0.2 (facts 11 and 12). The other bodies
  expand through the UGens and the operators that they call.
- `Sum3` and `Sum4` copy `insertionSortRange` (fact 13). The sort of Rust is
  stable, and gives another order.
- For `Wiring::audio_input`, `ugen` converts the whole argument before the
  expansion (fact 24). For `Wiring::level`, it multiplies the whole result
  by `level` after the expansion.

### The values and the errors

- `ArgValue` gains `Bool` and becomes `non_exhaustive`. A bool default binds
  as a `Bool`. A real UGen refuses a `Bool` with `WrongKind`.
- `BuildError::UnsupportedRate` names a pseudo-UGen and a rate that it does
  not take: `Mix.ar` and `Mix.kr` at the rates that they refuse (fact 15),
  and `blend` of a UGen at demand or scalar rate (fact 10).
- The other refusals reuse the errors of 0.2: `MissingArgument`,
  `WrongKind`, `BadCount`, `EmptyInput` and `NestedArray`.

### The arithmetic of the language

- Two numbers compute in f64, with the functions of fact 4, by the name of
  the operator. An array goes element by element, and the shorter array
  wraps (fact 5). Anything else goes to `binary_op` or `unary_op`, with
  their folds.
- `sum` starts from 0 (fact 6), and `rate` takes the least name (fact 7).
- `clip` and `moddif` of a UGen add a `Clip` and a `ModDif` (fact 9).
  `blend` and `linlin` come with `SplayAz` (fact 10).
- The private `operate` of 0.2 moves into this module.

## Differences from sclang that remain

The README lists them, with those of 0.1 and 0.2.

- A UGen in an array of buses of `SoundIn`: sclang builds a `+ 1` for its
  test of consecutive buses, and `optimizeGraph` removes it (fact 17). The
  builder refuses the UGen.
- An array of specs for `DynKlank` and `DynKlang`: sclang expands it
  (fact 12). The builder takes one spec.
- The `fill` methods, which take a function: the caller builds the array.
- `blend` of a UGen at scalar rate, in `SplayAz`: sclang builds a
  `LinXFade2` outside the graph (fact 10). The builder refuses it.
- The warnings of `Dstutter` and `Mix.kr`: the crate does no I/O, and builds
  the same graph.
- Numbers are Floats. sclang keeps Integers apart, but gives the same values
  for the operators of the bodies.

## Tests

- Golden files that sclang 3.14.1 writes with `generator/fixtures.scd`: one
  file for each pseudo-UGen, for the delays and for `LinXFade2`, with one
  definition for each case. The cases cover numbers and UGens, each rate
  method, each fold, the arrays that expand, and the branches of each body.
- For a graph that `optimizeGraph` rewrites, `tests/fixtures/before_optimize`
  holds the build without it (decision 4).
- `tests/fixtures/arithmetic.json` holds the operands and the results of the
  operators of the arithmetic, as sclang computes them.
- `tests/common` builds each golden graph with `ugen`. `tests/parity.rs`
  compares the bytes, and `tests/scsynth.rs` loads them in NRT mode.
- `tests/pseudo.rs` tests what the golden files do not show: the numbers
  that a pseudo-UGen gives, its errors, and a failed call that adds no UGen.
- Unit tests: the bodies against the catalogue, and the arithmetic against
  its table.

## Steps

One commit for each step, on the branch `feat/pseudo-ugens`. Each commit
passes the commands of CLAUDE.md.

1. docs: the plan of version 0.3.
2. test: the golden files of 0.3, written by sclang 3.14.1.
3. feat: pseudo-UGens by name, and the first wave: `PMOsc`, `Changed`,
   `Silent`, `Dstutter`, `TChoose`, `TWChoose`, `Tap`, `LinLin`, `SoundIn`,
   `SelectX`, `BLowPass4` and `BHiPass4`.
4. feat: `Sum3`, `Sum4` and `Mix`.
5. feat: the second wave: `NumChannels`, `SelectXFocus`, `MultiTap`,
   `Splay`, `DynKlank`, `DynKlang` and `UnpackFFT`.
6. feat: the audio-rate input of the delays, and `LinXFade2`.
7. feat: `CompanderD`, `HilbertFIR`, `LinSelectX` and `SplayAz`.
8. docs: the release of version 0.3. The README, the CHANGELOG, the version
   0.3.0, and this file trimmed to what is still open.

Then the branch reaches `main` through a pull request.

## After version 0.3

These stay open, in the order of their value for sc-app3:

1. nested arrays in `Array` arguments, as each class expands them
2. `optimizeGraph`, and `addCopiesIfNeeded` for PV chains
3. what rate methods do besides `new1`: the Lag family, `LocalBuf`,
   `LocalIn`, `Poll` and the zeros of `Out`
4. `NamedControl`, `Control.names`, `SynthDef.wrap`, `prependArgs` and the
   specs of the metadata
5. `Env`, and the pseudo-UGens that wait for items 3 to 6: `Dunique`,
   `PingPong`, `EnvGate`, and the `pvcalc` methods of `PV_ChainUGen`
6. the rules of `checkInputs` that classes define for themselves
7. sc3-plugins
