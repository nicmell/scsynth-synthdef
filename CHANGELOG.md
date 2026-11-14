# Changelog

## 0.5.0 (2026-11-14)

The rules of `checkInputs` that classes define for themselves, `Klank`,
`Klang` and `PingPong`.

### Added

- `CheckInputs::BufWr`, `Duty`, `Peak` and `T2K`: the rules of these
  classes, which their subclasses inherit. The 13 methods of `BufWr`,
  `Duty`, `TDuty`, `Peak`, `RunningMax`, `RunningMin` and `T2K` become
  addable.
- `Klank.ar` and `Klang.ar` have a body, with the spec of `DynKlank`: the
  triplets of the spec end the inputs of the UGen.
- `PingPong.ar` has a body. It is a pseudo-UGen of the catalogue now.
- `generator/fixtures.scd` writes the golden files of 0.5: `bufwr`, `duty`,
  `peak`, `t2k`, `klank`, `klang` and `pingpong`.

### Changed

- The 15 methods above no longer give `NotAddable`. `ugen` refuses 28
  methods, from 43. No class of the catalogue has `CheckInputs::Custom`.
- `PingPong.inputs` is an array.

## 0.4.0 (2026-11-14)

The rate methods of SuperCollider 3.14.1 that do more than `new1`, as
sclang builds them.

### Added

- The Lag family gives back its input in place of a UGen, as sclang does.
  `Lag`, `Lag2`, `Lag3` and `Ramp` do it for an input at scalar rate or a
  `lagTime` of 0. `LagUD`, `Lag2UD`, `Lag3UD` and `Slew` do it for an input
  at scalar rate. Their 16 methods become addable.
- `Out.ar`, `ReplaceOut.ar`, `OffsetOut.ar`, `LocalOut.ar` and `XOut.ar` put
  silence in place of a 0 channel, as `replaceZeroesWithSilence` does.
- `LocalBuf.new` adds the `MaxLocalBufs` of the build, and counts its
  buffers in it. `BeatTrack.kr`, `LocalIn.ar`, `LocalIn.kr`, `Poll.ar`,
  `Poll.kr` and `Poll.new` have a body. See
  [Methods that do more than `new1`](README.md#methods-that-do-more-than-new1).
- `Wiring::passthrough`, a `Passthrough`, and `Wiring::silence`.
- `generator/catalog.scd` takes the tables `passthroughs` and
  `silentZeros`, and probes test each entry. `generator/fixtures.scd`
  writes the golden files of 0.4: `lag`, `outzeros`, `localbuf`,
  `beattrack`, `localin`, `poll` and `symbols`.

### Changed

- The codes of a symbol argument, such as the name of `SendReply`, are its
  bytes as `Char:ascii` gives them in sclang, from -128 to 127. They were
  the bytes from 0 to 255.
- `LocalBuf.new` takes signals for `numFrames` and `numChannels`, and
  `LocalIn` takes an array for `default`.
- The 23 methods above no longer give `NotAddable`. `ugen` refuses 43
  methods, from 66.

### Fixed

- `fail` of both generators throws after `1.exit`. sclang 3.14.1 runs the
  code after `1.exit`, so a failed test went on, and the script ended
  with 0.

## 0.3.0 (2026-11-13)

The pseudo-UGens of SuperCollider 3.14.1, by class name, as sclang builds
them.

### Added

- `SynthDefBuilder::ugen` adds the pseudo-UGens by class name: `BHiPass4`,
  `BLowPass4`, `Changed`, `CompanderD`, `Dstutter`, `DynKlang`, `DynKlank`,
  `HilbertFIR`, `LinLin`, `LinSelectX`, `Mix`, `MultiTap`, `NumChannels`,
  `PMOsc`, `SelectX`, `SelectXFocus`, `Silent`, `SoundIn`, `Splay`,
  `SplayAz`, `TChoose`, `TWChoose`, `Tap` and `UnpackFFT`. A body of the
  crate adds the UGens of each method in the order of sclang, and computes
  with numbers as sclang does. See [Pseudo-UGens](README.md#pseudo-ugens).
- `Sum3.new` and `Sum4.new`, with the folds of `new1` and the sort of
  sclang.
- The 19 delay methods at audio rate and `LinXFade2` become addable:
  `Wiring::audio_input` and `Wiring::level`.
- The module `catalog` gains `PseudoClass`, `pseudo_lookup` and
  `pseudo_all` for the classes outside `UGen`, `Addable::Pseudo`, and
  `ArgKind::Bool`.
- `ArgValue::Bool`, for `levelComp`, `wrap` and `mixdown`.
- `BuildError::UnsupportedRate`, for `Mix.ar` and `Mix.kr` of a mix at a
  rate that they do not take, and for the `blend` of `SplayAz` at scalar
  or demand rate.
- `generator/catalog.scd` takes a census of the pseudo-UGens.
  `generator/fixtures.scd` writes 27 golden files of 0.3, six of them also
  without `optimizeGraph` in `tests/fixtures/before_optimize`, and the
  operators of sclang on Floats in `tests/fixtures/arithmetic.json`.

### Changed

- `ArgValue` is `non_exhaustive`, and gains `Bool`. A match on it needs a
  wildcard arm.
- A bool default binds as an `ArgValue::Bool`, and a real UGen refuses a
  `Bool` with `WrongKind`.
- The 21 methods of the delays at audio rate and of `LinXFade2` no longer
  give `NotAddable`.

## 0.2.0 (2026-11-08)

Multichannel expansion, `madd`, and the folds of the operators, as sclang
3.14.1 applies them.

### Added

- Multichannel expansion in `SynthDefBuilder::ugen`, `unary_op` and
  `binary_op`, as `multiNewList` expands: the arguments of the wire and the
  count of the outputs expand, depth first, and the shorter arrays wrap. See
  [Multichannel expansion](README.md#multichannel-expansion).
- `SynthDefBuilder::madd`: `MulAdd.new`, with the cases of `MulAdd:new1`.
  `ugen` applies `mul` and `add` through it, once on the whole expansion.
  See [`madd` and the folds](README.md#madd-and-the-folds).
- `BuildError::EmptyInput` and `BuildError::NestedArray`, for the two
  inputs that the builder refuses.
- `Wiring::nested_expansion`. `generator/catalog.scd` measures the
  expansion of every addable method.
- 13 golden files of the expansion and of `madd`. scsynth loads them in the
  acceptance test.

### Changed

- `binary_op` folds as `BinaryOpUGen:new1` does, so it can give back a
  number or one of its inputs instead of a new output. A product with 0
  then reaches `Out.ar` as the number 0, which the rule of `AbstractOut`
  refuses. Version 0.1 built a `BinaryOpUGen` there.
- `ugen`, `unary_op` and `binary_op` expand an array instead of refusing
  it. Only `add` still gives `BuildError::MultichannelExpansion`.
- `ugen` applies `mul` and `add` instead of refusing them.
  `BuildError::MaddNotSupported` now refuses them only on a UGen without
  outputs.

## 0.1.0 (2026-10-18)

The first release. It compiles synth definitions as sclang 3.14.1 compiles
them. It reads and writes their files. It carries a catalogue of the UGens
of the core class library.

### Added

- `read_def_file`, `write_def_file`, `write_def_file_old` and
  `SynthDef::as_bytes`: definition files of versions 0, 1 and 2, as Rust
  types. Reading is strict, and both directions apply the limits of
  scsynth. See [Definition files](README.md#definition-files) and
  [Strictness](README.md#strictness).
- `SynthDefBuilder`: the build of sclang without `optimizeGraph`. Controls
  from the arguments, `add`, variants, the constants and the topological
  sort of sclang. See [Building](README.md#building) and
  [Differences from sclang](README.md#differences-from-sclang).
- `SynthDefBuilder::unary_op` and `binary_op`, and the module `operator`:
  the operators of scsynth by name and by special index, generated from
  the sources of SuperCollider 3.14.1.
- The module `catalog`: the 419 UGen classes of the core class library of
  SuperCollider 3.14.1, with their arguments, their categories, the rules
  of their `checkInputs`, and the wiring of each rate method that `ugen`
  can add. See [The catalogue](README.md#the-catalogue).
- `SynthDefBuilder::ugen` and `ArgValue`: a UGen by its class name, with
  the arguments by name, the defaults and the wiring of the catalogue, and
  the checks of `checkInputs`.
- The feature `serde`, for the types of a definition and of a file.

### Known issues

- See [Left out](README.md#left-out) for what version 0.1 does not do yet,
  and [Traps](README.md#traps) for what it does not rule out.
