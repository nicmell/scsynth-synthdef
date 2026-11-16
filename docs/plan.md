# The plan of scsynth-synthdef

This document is the plan of version 0.11. A session starts from it and
from the README, in this folder. The plan owns its decisions and facts
until the code and the tests own them. Then the facts move to the tests,
the README and the CHANGELOG, and this file keeps only what is still open.

## What we build

Version 0.11 takes item 1 of "Next": `PackFFT.new`, whose inputs follow a
count, and the `pvcalc` methods of `PV_ChainUGen`. Today a graph can
unpack an FFT chain with `UnpackFFT`, but `ugen` refuses `PackFFT.new`, so
the graph cannot pack the chain again. The `pvcalc` methods take a
function, which no method of the crate takes yet.

- Each graph with `PackFFT.new` or a `pvcalc` method writes the bytes that
  sclang 3.14.1 writes.
- Where sclang throws, the crate refuses with a `BuildError`.

Then sc-app3 takes the release. Its markup reaches `PackFFT` with
`UnpackFFT` and expressions, but not the `pvcalc` methods, because the
markup has no functions.

- The crate stays a library for any Rust user. Nothing of sc-app3 enters it.
- The crate keeps its rules: no I/O, serde as its only optional dependency,
  the MSRV 1.85, the `wasm32-unknown-unknown` build, its lints, and docs on
  every public item.

## Ground rules

- This is a clean implementation from the SuperCollider sources. Do not
  consult earlier compilers: neither `sc-app2/packages/synthdef-compiler`
  nor `sc-crates/crates/scsynthdef-compiler`. No search of sc-app3 reaches
  its submodule `sc-app2`.
- Where sclang writes a definition that is broken, or throws, the crate
  refuses the input.
- A crate item lands with its first caller, so that no commit warns about
  dead code.
- Never touch the scsynth on UDP 57110.

## Decisions

The owner took these on 2026-11-16.

1. `PackFFT.new` gets a body of the crate, through `ugen`, as `VarLag` got
   one in 0.9. Its count of `magsphases` and its default `tobin` are
   computed, so no rule of the catalogue wires it.
2. `SynthDefBuilder::pvcalc`, `pvcalc2` and `pvcollect` take the chain,
   the arguments by name with the defaults of sclang from the catalogue,
   and a closure in place of `func`. A new table `pv_methods` holds them,
   as `env_methods` holds the methods of `Env`.
3. The crate copies every quirk of sclang 3.14.1. It refuses what sclang
   throws on, with a `BuildError`.
4. sc-app3 takes 0.11 with the move of its submodule and one byte test,
   through `<sc-ugen type="PackFFT">`.
5. After 0.11 come the methods of `UGen` that build graphs, then
   sc3-plugins.
6. When the CI of the PR is green, the work stops. On the go of the owner,
   the PR merges, and `v0.11.0` becomes an annotated tag on its merge
   commit. Then sc-app3 moves, and its PR merges into `develop` once its CI
   is green.

## Sources of truth

The SuperCollider source at the tag `Version-3.14.1`, in
`~/Projects/supercollider`, and sclang and scsynth 3.14.1. In
`SCClassLibrary/Common/Audio`: `FFTUnpacking.sc`.

## Facts established on 2026-11-16

The measured facts come from sclang 3.14.1, with probes of one line on
stdin.

### `PackFFT.new`

1. `PackFFT.new(chain, bufsize, magsphases, frombin = 0, tobin,
   zeroothers = 0)` is `multiNewList` of the rate `control`, the chain,
   `bufsize`, `frombin`, `tobin`, `zeroothers`, `magsphases.size`, then
   each element of `magsphases`. A nil `tobin` becomes `bufsize / 2`.
   `fftSize` is input 1.
   - With `bufsize` 8, the default `tobin` is 4.0. With `bufsize:
     \b.kr(8)`, the `/` adds a `BinaryOpUGen` before the `PackFFT`.
   - An output as `frombin` builds. `magsphases: []` writes the size 0.
   - An array of chains makes two `PackFFT`. `magsphases: [[1, 2], 0]`
     makes two as well, each with the size 2. The chain then has two
     readers that are width-first. So a `PV_Copy`, with its `BufFrames` and
     its `LocalBuf`, comes before the first.
2. `UnpackFFT` clamps `tobin` to `bufsize / 2`, and `PackFFT` does not.
   The crate copies `UnpackFFT` already.

### The `pvcalc` methods

3. `pvcalc(numframes, func, frombin = 0, tobin, zeroothers = 0)` unpacks
   the chain, as `UnpackFFT(this, numframes, frombin, tobin).clump(2).flop`.
   It calls `func` with the magnitudes and the phases.
   - Then it switches on the `size` of the result. A size of 1 appends the
     phases, as `result ++ phases`. A size of 2 is `[mags, phases]`. Any
     other size, the 0 of a UGen included, gives `[result, phases]`.
   - Then `flop`, and `flatten` of one level. Last, it calls
     `PackFFT(this, numframes, values, frombin, tobin, zeroothers)` with
     `tobin` as given.
   - Measured: the identity `[m, p]`, `m * 2`, and one bin, with `frombin`
     and `tobin` 2. Also `m[0]`, a UGen, which takes the place of every
     magnitude.
   - Measured: two bins with magnitudes alone count as `[mags, phases]`, so
     `PackFFT` gets two values. The two `Unpack1FFT` of the phases stay,
     after the `Out`.
   - Measured: `tobin` 10 unpacks bins 0 to 4, and `PackFFT` takes 10.
4. `pvcalc2(chain2, numframes, func, ...)` unpacks the chain, then
   `chain2`. It calls `func(m, p, m2, p2)`, switches as `pvcalc` does, and
   packs the chain. The unused `Unpack1FFT` of `chain2` stay, after the
   `Out`.
5. `pvcollect(numframes, func, ...)` calls `func(mag, phase, index +
   frombin, index).asArray` for each bin. A result of one value gets the
   phase of its bin, and more values stay. Measured: three values for five
   bins give 15. Then `flatten`, and `m * bin` folds at the bin 1.
6. sclang throws on these:
   - `numframes` as an output: "Primitive '_SimpleNumberSeries' failed".
   - `frombin` as an output of `pvcollect`: "'prSimpleNumberSeries' not
     understood".
   - `pvcalc` of a value that is no PV chain, such as `SinOsc.ar`, and of
     an array of chains: "Message 'pvcalc' not understood".
7. The instance methods of `PV_ChainUGen` are `pvcalc`, `pvcalc2` and
   `pvcollect`, which take a function, and also `addCopiesIfNeeded` and
   `fftSize`. The defaults are `frombin` 0, `tobin` nil and `zeroothers` 0.

## Version 0.11

### `PackFFT.new`

- A body `pack_fft` takes `tobin`, or `bufsize / 2` through `lang::binary`.
  So a number folds, and an output adds its UGen. Then the body expands
  with `builder.expand`, as `multiNewList` does.
- Each instance adds a `PackFFT` at control rate, with its `checkInputs`
  and `width_first` from the catalogue.
- `generator/catalog.scd` names `PackFFT` among the classes with a body.

### The `pvcalc` methods

- The table `pv_methods` comes from the methods of `PV_ChainUGen`. Each
  other instance method needs a reason in a table, or the script fails.
  The kind `function` marks `func`, and `ArgKind` gains `Function`.
  `catalog::pv_method` and `catalog::pv_methods` give the table.
- `pvcalc` takes a closure of the magnitudes and the phases, each as an
  array. `pvcalc2` takes the four arrays. `pvcollect` takes a closure of
  the magnitude, the phase, the bin and the index, the last two as `f64`.
- The bodies reuse the body of `UnpackFFT` and `pack_fft`, and copy the
  switch and the `flop` of facts 3 to 5. The `flop` of `splay.rs` moves to
  `lang.rs`.
- A chain that is not one output of a `PV_ChainUGen` gives `WrongKind`, and
  so does a value for `func` by name. `numframes`, `frombin` and `tobin` as
  outputs keep the errors of the body of `UnpackFFT`.

## Tests

- `generator/fixtures.scd` writes two files.
  - `packfft` holds the cases of fact 1, and `packfft_markup`, the graph of
    the test of sc-app3.
  - `pvcalc` holds the cases of facts 3 to 5, each with an FFT of 8 frames.
- Each refused case of fact 6 must break in sclang, and a test of the crate
  expects its error.
- `tests/golden.rs` counts 89 files. `tests/fixtures/wire_buffers.json`
  gains the two files in step 2, because the test of the counts reads
  every golden file. `NAMES` gains each file in its step.
- `tests/readme.rs` counts 18 refused methods. A test of the catalogue
  owns the table `pv_methods`.
- `tests/scsynth.rs` loads the new graphs.

## Steps

Each step is one commit on `feat/pack-fft`. Every commit leaves the eight
commands of `CLAUDE.md` green.

1. `docs: the plan of version 0.11`.
2. `test: the golden files of 0.11, written by sclang 3.14.1`.
3. `feat: PackFFT.new`: the body, the census, `flop` in `lang.rs`, the
   parity of `packfft`, and the refused list of the README.
4. `feat: the pvcalc methods`: the table `pv_methods`, `ArgKind::Function`,
   the three methods, the parity of `pvcalc`, and the tests of the
   refusals.
5. `docs: the release of version 0.11`.

## sc-app3

After the tag, sc-app3 takes `v0.11.0` on a branch
`build/scsynth-synthdef-0.11.0` from `origin/develop`, in a worktree of
its own. The submodule moves, and `Cargo.lock` changes one line. Core's
definitions keep their bytes. A new test of sc-expression compiles the
markup of `packfft_markup` against the golden file of the crate. The PR
goes to `develop`, and merges once its CI is green.

## After version 0.11

The work after version 0.11, in the order of its value for sc-app3:

1. The methods of `UGen` that build graphs, such as `range`, `exprange` and
   `linlin`, which need the `signalRange` of each class, and with them
   `range`, `exprange` and `curverange` of `Env`
2. sc3-plugins
