# The plan of scsynth-synthdef

This document is the plan of version 0.4. A session starts from it and from
the README, in this folder. The plan owns its decisions and facts until the
code and the tests own them. Then the facts move to the tests, the README
and the CHANGELOG, and this file keeps only what is still open.

## What we build

Version 0.4 is item 3 of the list "Next" of version 0.3. It adds the rate
methods of the core class library that do more than `new1`.

- The Lag family returns its input, in place of a UGen, under the condition
  of sclang.
- `LocalBuf.new` adds the `MaxLocalBufs` of the definition, and counts its
  buffers in it.
- `LocalIn`, `Poll` and `BeatTrack.kr` get a body, as the pseudo-UGens of
  version 0.3 have.
- The `.ar` methods of `Out`, `LocalOut` and `XOut` replace a 0 with
  silence.

Then sc-app3 takes the release, as it took 0.3.

- The crate stays a library for any Rust user. Nothing of sc-app3 enters it.
- The crate keeps its rules: no I/O, serde as its only optional dependency,
  the MSRV 1.85, the `wasm32-unknown-unknown` build, its lints, and docs on
  every public item.

## Ground rules

- This is a clean implementation from the SuperCollider sources. Do not
  consult earlier compilers: neither `sc-app2/packages/synthdef-compiler`
  nor `sc-crates/crates/scsynthdef-compiler`. No search of sc-app3 reaches
  its submodule `sc-app2`.
- Where sclang writes a definition that scsynth refuses, the crate refuses
  the input.
- A crate item lands with its first caller, so that no commit warns about
  dead code.
- Never touch the scsynth on UDP 57110.

## Decisions

The owner took these on 2026-11-14.

1. Version 0.4 takes item 3 of "Next", before item 1. Item 3 makes 23 of the
   66 refused methods addable. Item 1 refuses no method of the catalogue.
2. The plan goes to the owner as a plan, with no artifact.
3. The plan includes the move of sc-app3 to the tag `v0.4.0`.
4. When the CI of the PR is green, the work stops. On the go of the owner,
   the PR merges with a merge commit, and `v0.4.0` becomes an annotated tag
   on that commit. Then sc-app3 moves.

The planning found these limits of the item:

- `VarLag` builds an `Env` with its default warp (fact 4). It waits for
  item 5.
- `LagControl` is a control. It waits for item 4.
- `Dunique` builds a reader in `asUGenInput`, at each use as an input (fact
  10). In the crate, a method adds all its UGens at its call. So `Dunique`
  stays out.
- `PingPong` needs `BufWr`, whose rule is item 6.
- `BeatTrack.kr` enters the item: it builds a `LocalBuf` (fact 9).

## Sources of truth

The SuperCollider source at the tag `Version-3.14.1`, in
`~/Projects/supercollider`, and sclang and scsynth 3.14.1:

- `SCClassLibrary/Common/Audio`: `Filter.sc`, `BufIO.sc`, `InOut.sc`,
  `Poll.sc`, `MachineListening.sc`, `FFT.sc`, `Demand.sc`, `DelayWr.sc`,
  `UGen.sc` and `SynthDef.sc`.
- `SCClassLibrary/Common/Collections/Array.sc` and
  `Common/Math/SimpleNumber.sc`: `madd`.

sclang runs with `-l generator/sclang_conf.yaml`, the core class library
alone. A probe wraps its body in `try`, with `0.exit` at its end and `1.exit`
in its handler. scsynth runs from the bundle with `-U`, in NRT mode, as
`tests/scsynth.rs` runs it.

## Facts established on 2026-11-14

All of them come from the sources.

### The Lag family

1. If `(in.rate == \scalar) || (lagTime == 0)`, `Lag.ar` and `Lag.kr`
   (`Filter.sc:70-86`) return `in.madd(mul, add)`. Else they return
   `multiNew(...).madd(mul, add)`. The test runs before `multiNew`. `Lag2`,
   `Lag3` and `Ramp` inherit both methods.
2. `LagUD` (`Filter.sc:92-108`), which `Lag2UD` and `Lag3UD` inherit, and
   `Slew` (`Filter.sc:281-295`) test only `in.rate == \scalar`.
3. `SimpleNumber:madd` is `(this * mul) + add` (`SimpleNumber.sc:597`).
   `Array:madd` and `UGen:madd` are `MulAdd(this, mul, add)` (`Array.sc:281`,
   `UGen.sc:51`).
4. `VarLag.new1` builds `Env([start, in], [time], warp).asArray` and an
   `EnvGen`, unless the warp is linear (`Filter.sc:113-147`).

### `LocalBuf` and `LocalIn`

5. `LocalBuf.new` calls `multiNew('scalar', numChannels, numFrames)`
   (`BufIO.sc:144-158`). Its `new1` takes the `MaxLocalBufs` of the build,
   and makes it at the first `LocalBuf`. Each build starts without one
   (`SynthDef.sc:81`). `new1` adds 1 to input 0 of the `MaxLocalBufs`, then
   adds the `LocalBuf`, with the inputs
   `[numChannels, numFrames, maxLocalBufs]`. `LocalBuf` is a
   `WidthFirstUGen` (`FFT.sc:4`), and `MaxLocalBufs` is not
   (`BufIO.sc:182-189`).
6. `LocalIn` calls `multiNew(rate, numChannels, *default)`
   (`InOut.sc:182-192`). Its inputs are `default.wrapExtend(numChannels)`,
   and it has `numChannels` outputs.

### The zeros of the outputs

7. `Out.ar`, `LocalOut.ar` and `XOut.ar` call
   `replaceZeroesWithSilence(channelsArray.asUGenInput(this).asArray)` before
   `multiNewList` (`InOut.sc:257, 276, 291`). `ReplaceOut` and `OffsetOut`
   inherit `Out.ar`. The `kr` methods replace nothing.
   `replaceZeroesWithSilence` (`UGen.sc:407-429`) counts the zeros of the top
   level. If there are none, it gives the array back at once. Else it makes
   one `Silent.ar(numZeroes)`, puts its channels in place of the zeros, and
   repeats this in each array inside the array.

### `Poll` and `BeatTrack`

8. The rate methods of `Poll` call
   `multiNewList([rate, trig, in, label, trigid])` and return `in`
   (`Poll.sc:1-30`). `Poll.new` takes its rate from `in`. `Poll.new1` does
   these things:
   - A nil label becomes `"UGen(%)".format(in.class)`, and the label becomes
     its ASCII codes.
   - A scalar rate becomes control.
   - A number trigger becomes `Impulse.multiNew(rate, trig, 0)`.
   - The inputs are `[trig, in, trigid, label.size] ++ label`.
   - Its `checkInputs` is `checkSameRateAsFirstInput`.
9. Unless `chain.isKindOf(FFT)`, `BeatTrack.kr` makes the chain
   `FFT(LocalBuf(if(SampleRate.ir > 48000, 2048, 1024)), chain)`
   (`MachineListening.sc:2-16`). `UGen:if` is
   `(this * (trueUGen - falseUGen)) + falseUGen` (`UGen.sc:456`). Then it
   calls `multiNew('control', chain, lock)`, with 4 outputs.

### What stays out

10. `Dunique.new` builds a writer, and `Dunique:asUGenInput` builds a reader
    at each use (`Demand.sc:199-235`). `PingPong.ar` adds a `BufWr.ar`
    (`DelayWr.sc:1-19`).

### The generator and the builder

11. To wire a method, the probe of `generator/catalog.scd` needs one
    instance of the class from numbers, and one from control outputs. With
    numbers, the Lag family makes none.
12. The builder collects the constants in `finish`, in the order of
    creation. `transaction` restores `children` and `width_first`, and
    nothing else. `add_instance` applies the rule of `checkInputs` and the
    width-first flag of the class.

## Version 0.4

### The catalogue

- A table `passthroughs` in `generator/catalog.scd` names the methods of
  facts 1 and 2: the owner, the selector, the input, and the argument whose
  0 also returns the input, or nil. `Lag` takes `lagTime`. `LagUD` and
  `Slew` take nil. The input gets the kind `through`, and the probe makes it
  a control output inside the build. So the probe wires the case that makes
  an instance.
- Three probes test the table. If one fails, the generator stops. A number
  input makes no UGen. With an argument of 0, a control input and
  that 0 make no UGen. Without one, a control input and a 0 in each other
  signal make one instance.
- A table `silentZeros` names the methods of fact 7: `Out`, `LocalOut` and
  `XOut` at `ar`, on `channelsArray`. A probe tests it: a 0 in the array
  makes one `DC` at audio rate before the instance, and the instance takes
  its output.
- `bodies` gains `BeatTrack`, `LocalBuf`, `LocalIn` and `Poll`. `notPseudo`
  loses the 8 classes of the Lag family. `LagControl`, `VarLag`, `Dunique`
  and `PingPong` take the reasons of the decisions.
- `LocalIn.default` becomes an array at both rates, as `multiNew` takes it
  apart. `LocalBuf.numFrames` and `numChannels` become signals: they are
  plain inputs, and `BeatTrack` gives `numFrames` a UGen.
- `Wiring` gains `passthrough`, a `Passthrough` with the positions of the
  input and of its zero argument, and `silence`, the position of the
  argument whose zeros become silence. `Addable::Pseudo` covers the methods
  with a body, as for `Sum3.new` in version 0.3.

### The builder

- On a method with a `passthrough`, `ugen` tests the condition of fact 1
  before the expansion. When it holds, `ugen` gives `in.madd(mul, add)`: for
  a number, `*` and then `+` of the language, and else `MulAdd.new`.
- On a method with a `silence`, `ugen` puts the argument through
  `replaceZeroesWithSilence` before the expansion. The `DC` comes from the
  body of `Silent`.
- The builder keeps the `MaxLocalBufs` of the build and its count. The first
  `LocalBuf` adds it, with a placeholder for input 0. `transaction` restores
  the count, and `finish` writes it into input 0 before it collects the
  constants.
- `class_of` gives the class of the UGen of an output, for `BeatTrack` and
  the label of `Poll`.

### The bodies

- `src/pseudo/buf_io.rs`: `LocalBuf.new` expands over `numChannels` and
  `numFrames`. Each instance takes the `MaxLocalBufs` and adds a
  width-first `LocalBuf`.
- `src/pseudo/machine_listening.rs`: `BeatTrack.kr` adds the `FFT` of fact
  9, unless the chain comes from an `FFT`.
- `src/pseudo/in_out.rs`: `LocalIn` expands over its count and each element
  of its default. A count under 1 and an empty default are refused.
- `src/pseudo/poll.rs`: `Poll` expands over `trig`, `in` and `trigid`. Each
  instance takes its rate, its label, its `Impulse` and its rule, as in
  fact 8. The call returns `in`.
- The codes of a label are those of a symbol argument, from one function.

## Differences from sclang that remain

- The default label of `Poll` for a whole number is "UGen(Integer)". sclang
  writes "UGen(Float)" for a Float such as 440.0.
- `Poll` takes one label. sclang expands an array of labels, and `ArgValue`
  has no array of texts.

## Tests

- `generator/fixtures.scd` writes six files: `lag`, `outzeros`, `localbuf`,
  `beattrack`, `localin` and `poll`. `poll` has a label with a byte of 128 or
  more, for the codes of sclang.
- `tests/common/pseudo.rs` builds each graph. `NAMES` grows at each step, so
  `tests/golden.rs`, `tests/parity.rs` and `tests/scsynth.rs` cover it.
- `tests/pseudo.rs` tests what golden files do not show. `Lag.kr` of an
  array of a control and a number fails `checkInputs`, as in sclang.
  `LocalIn` with 0 channels or an empty default is refused. `Poll.kr` with
  an audio trigger fails `checkInputs`. A failed call leaves the count of
  `MaxLocalBufs`. `Out.kr(0, 0)` adds no `DC`.
- The tests of the catalogue count 16 methods with a passthrough and 5 with
  a silence. `tests/readme.rs` counts 43 refused methods.

## Steps

Each step is one commit on `feat/rate-methods`. Every commit leaves the
eight commands of `CLAUDE.md` green.

1. `docs: the plan of version 0.4`.
2. `test: the golden files of 0.4, written by sclang 3.14.1`.
3. `feat: the Lag family returns its input for a scalar or a lag of 0`.
4. `feat: the zeros of Out, LocalOut and XOut become silence`.
5. `feat: LocalBuf and BeatTrack`.
6. `feat: LocalIn and Poll`.
7. `docs: the release of version 0.4`.

Steps 3, 5 and 6 update "Methods that `ugen` refuses" in the README and the
count of `tests/readme.rs`.

## sc-app3

After the tag, and after PR #104 of sc-app3 merges into `develop`, sc-app3
takes `v0.4.0` on a branch `build/scsynth-synthdef-0.4.0` from `develop`.
The submodule moves, and `Cargo.lock` changes one line. Core's definitions
keep their 2,301 bytes, measured on both tags. New tests of sc-expression
compare a markup `Lag.kr` and an `Out.ar` of 0 with the golden files of the
crate. `docs/synthdefs.md` records the move. The PR goes to `develop`.

## After version 0.4

The work after version 0.4, in the order of its value for sc-app3:

1. nested arrays in array arguments, as each class expands them
2. `optimizeGraph`, and `addCopiesIfNeeded` for PV chains. Each file of
   `tests/fixtures/before_optimize` and the file of its name in
   `tests/fixtures` are a graph before and after `optimizeGraph`.
3. `NamedControl`, `Control.names`, `SynthDef.wrap`, `prependArgs`, the
   specs of the metadata, and `LagControl`
4. `Env`, `VarLag`, and the pseudo-UGens that wait for items 3 and 5:
   `EnvGate`, and the `pvcalc` methods of `PV_ChainUGen`
5. the rules of `checkInputs` that classes define for themselves, and
   `PingPong`, which needs the rule of `BufWr`
6. sc3-plugins
