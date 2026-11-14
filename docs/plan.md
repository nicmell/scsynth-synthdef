# The plan of scsynth-synthdef

This document is the plan of version 0.5. A session starts from it and from
the README, in this folder. The plan owns its decisions and facts until the
code and the tests own them. Then the facts move to the tests, the README
and the CHANGELOG, and this file keeps only what is still open.

## What we build

Version 0.5 takes the cheap half of the 43 methods that `ugen` refuses after
version 0.4:

- The rules of `checkInputs` that `BufWr`, `Duty`, `Peak` and `T2K` define
  for themselves, item 5 of "Next". With the classes that inherit them,
  they make 13 methods addable.
- `Klank.ar` and `Klang.ar`, with the one spec of `DynKlank`.
- `PingPong.ar`, a pseudo-UGen that needs `LocalIn` and the rule of
  `BufWr`.

Then sc-app3 takes the release, and its core adds `BufWr` through `ugen`.

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

1. Version 0.5 takes item 5 of "Next", with `Klank`, `Klang` and `PingPong`.
   Items 1 to 4 wait.
2. The plan goes to the owner as a plan, with no artifact.
3. When the CI of the PR is green, the work stops. On the go of the owner,
   the PR merges, and `v0.5.0` becomes an annotated tag on its merge commit.
   Then sc-app3 moves, and its PR merges into `develop` once its CI is
   green.

## Sources of truth

The SuperCollider source at the tag `Version-3.14.1`, in
`~/Projects/supercollider`, and sclang and scsynth 3.14.1:
`SCClassLibrary/Common/Audio/BufIO.sc`, `Demand.sc`, `Trig.sc`, `Line.sc`,
`FSinOsc.sc`, `DelayWr.sc` and `UGen.sc`, `Common/Core/AbstractFunction.sc`,
and `lang/LangPrimSource/PyrArrayPrimitives.cpp`.

## Facts established on 2026-11-14

All of them come from the sources.

### The rules of their own

1. `BufWr:checkInputs` (`BufIO.sc:71-83`): at audio rate, input 1, the
   phase, runs at audio rate, and so do inputs 3 and after, the channels.
2. `Duty:checkInputs` (`Demand.sc:15-24`), which `TDuty` inherits: if input
   0, `dur`, runs at demand rate, input 1, `reset`, runs at demand rate, at
   scalar rate, or at the rate of the UGen. Else any input is valid.
3. `Peak:checkInputs` (`Trig.sc:82-89`), which `RunningMax` and `RunningMin`
   inherit: at control rate, input 0 at audio rate is valid. Else input 0
   has the rate of the UGen.
4. `T2K:checkInputs` (`Line.sc:84-90`): input 0 runs at audio rate.

### `Klank` and `Klang`

5. `Klank.ar` (`FSinOsc.sc:45-60`) calls `multiNewList` over `input`,
   `freqscale`, `freqoffset` and `decayscale`. It does not expand the spec.
   Its `new1` builds
   `[freqs, amps ?? fill(freqs.size, 1.0), times ?? fill(freqs.size, 1.0)].flop.flat`.
   The inputs of the UGen are `[input, freqscale, freqoffset, decayscale]`
   and then that array.
6. `Klang.ar` (`FSinOsc.sc:22-40`) calls `multiNewList` over `freqscale` and
   `freqoffset`, with the spec `[freqs, amps, phases]`. A missing `amps` is
   ones, and a missing `phases` is zeros. The inputs are
   `[freqscale, freqoffset]` and then the triplets.

### `PingPong`

7. `PingPong.ar(bufnum = 0, inputs, delayTime, feedback = 0.7, rotate = 1)`
   (`DelayWr.sc:1-19`) runs these lines, in this order, and returns the
   delayed signals:
   - `max(0, delayTime * SampleRate.ir - ControlDur.ir).round`
   - `BufFrames.kr(bufnum)`
   - `Phasor.ar(0, 1, 0, frames)`
   - `LocalIn.ar(inputs.size) * feedback`
   - `BufRd.ar(inputs.size, bufnum, (phase - delaySamps).wrap(0, frames), 0)`
   - `LocalOut.ar(delayedSignals)`
   - `BufWr.ar((inputs + feedbackChannels).rotate(rotate) <! delayedSignals.asArray.first, bufnum, phase, 1)`

   A single UGen as `inputs` has a size of 0 in sclang. Measured: sclang
   then throws "a LocalIn: numChannels must be a nonzero positive integer,
   but received (0)".
8. `UGen:wrap(lo, hi)` adds a `Wrap` at the rate of the UGen
   (`UGen.sc:104-110`). `round` with no argument rounds to 1.0. `<!` is the
   binary operator `firstArg` (`AbstractFunction.sc:153`).
   `Array:rotate(n)` takes an Integer, and moves element i to
   `sc_mod(i + n, size)` (`PyrArrayPrimitives.cpp:1570`).

### The generator

9. `checkRules` of `generator/catalog.scd` marks `BufWr`, `Duty`, `Peak`
   and `T2K` as `\custom`. So their methods get the reason "the checkInputs
   of X is a rule of its own", and the wiring probe never runs on them.

## Version 0.5

### The catalogue

- `checkRules` gives the four classes named rules: `\buf_wr`, `\duty`,
  `\peak` and `\t2k`. The wiring probe then measures their 13 methods.
- `CheckInputs` gains `BufWr`, `Duty`, `Peak` and `T2K`. Each takes the
  name of its class, as `AbstractOut` does, and its doc states the rule.
  `Custom` stays for a later class.
- `bodies` gains `Klank`, `Klang` and `PingPong`, and `leftOut` loses
  `PingPong`. `PingPong.inputs` is an array.

### The rules and the bodies

- `src/check_inputs.rs` gets the four rules of facts 1 to 4, in the phrasing
  of its other messages.
- `src/pseudo/fsin_osc.rs`: `Klank.ar` and `Klang.ar` expand as `DynKlank`
  and `DynKlang` do. Each instance adds one UGen, with the triplets of facts
  5 and 6. The parts wrap to the longest, as `flop` does. A part that is
  empty, while another is not, is refused.
- `src/pseudo/delay_wr.rs`: `PingPong.ar` follows fact 7, line by line. It
  refuses a single input, and a `rotate` that is not a whole number.

## Tests

- `generator/fixtures.scd` writes seven files: `bufwr`, `duty`, `peak`,
  `t2k`, `klank`, `klang` and `pingpong`.
- `tests/common/pseudo.rs` builds each graph. `NAMES` grows at each step,
  so `tests/golden.rs`, `tests/parity.rs` and `tests/scsynth.rs` cover it.
- The rules refuse `BufWr.ar` with a control phase or a control channel,
  `Duty.kr` on a demand `dur` with an audio reset, and `Peak.ar` and
  `T2K.kr` of a control input. `PingPong` refuses a single input, and a
  `rotate` of 0.5.
- `tests/readme.rs` counts 28 refused methods.

## Steps

Each step is one commit on `feat/rules-of-their-own`. Every commit leaves
the eight commands of `CLAUDE.md` green.

1. `docs: the plan of version 0.5`.
2. `test: the golden files of 0.5, written by sclang 3.14.1`.
3. `feat: the rules of BufWr, Duty, Peak and T2K`.
4. `feat: Klank and Klang`.
5. `feat: PingPong`.
6. `docs: the release of version 0.5`.

Steps 3 to 5 update "Methods that `ugen` refuses" in the README and the
count of `tests/readme.rs`.

## sc-app3

After the tag, sc-app3 takes `v0.5.0` on a branch
`build/scsynth-synthdef-0.5.0` from `develop`. The submodule moves, and
`Cargo.lock` changes one line. Its core adds `BufWr` through `ugen`, and
its definitions keep their 2,301 bytes, measured on both tags. A new test
of sc-expression compares a markup `Klank` with the golden file of the
crate. The PR goes to `develop`, and merges once its CI is green.

## After version 0.5

The work after version 0.5, in the order of its value for sc-app3:

1. nested arrays in array arguments, as each class expands them
2. `optimizeGraph`, and `addCopiesIfNeeded` for PV chains. Each file of
   `tests/fixtures/before_optimize` and the file of its name in
   `tests/fixtures` are a graph before and after `optimizeGraph`.
3. `NamedControl`, `Control.names`, `SynthDef.wrap`, `prependArgs`, the
   specs of the metadata, and `LagControl`
4. `Env`, `VarLag`, and the pseudo-UGens that wait for items 3 and 4:
   `EnvGate`, and the `pvcalc` methods of `PV_ChainUGen`
5. sc3-plugins
