# The plan of scsynth-synthdef

This document is the plan of version 0.7. A session starts from it and from
the README, in this folder. The plan owns its decisions and facts until the
code and the tests own them. Then the facts move to the tests, the README
and the CHANGELOG, and this file keeps only what is still open.

## What we build

Version 0.7 takes item 1 of "Next": `optimizeGraph`, and
`addCopiesIfNeeded` for PV chains. These are the two steps of the
`finishBuild` of sclang that `SynthDefBuilder::finish` skips. So the crate
writes `x * 0.5 + 0.25` as a `*` and a `+`, where sclang writes one
`MulAdd`. The crate also keeps an unread `SinOsc`, which sclang removes.

- `finish` runs the whole `finishBuild`, so each graph that the crate
  writes has the bytes that sclang 3.14.1 writes.
- Where sclang writes a broken definition or throws, the crate refuses with
  `BuildError::FinishBuild`.

Then sc-app3 takes the release, so that an expression such as
`osc * amp + 0.5` becomes a `MulAdd`, as in sclang.

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

The owner took these on 2026-11-14.

1. `finish` always optimizes, as sclang does. The folder
   `tests/fixtures/before_optimize` goes, with the build without
   `optimizeGraph` in `generator/fixtures.scd`.
2. The crate copies the bookkeeping of readers of sclang exactly, full
   arrays included. It refuses each graph that sclang 3.14.1 breaks or
   fails on, with a new `BuildError`.
3. Two refusals stay for a later version, although dead code explains
   them: `mul` and `add` on a UGen without outputs, and an output in the
   buses of `SoundIn`.
4. When the CI of the PR is green, the work stops. On the go of the owner,
   the PR merges, and `v0.7.0` becomes an annotated tag on its merge commit.
   Then sc-app3 moves, and its PR merges into `develop` once its CI is
   green.

## Sources of truth

The SuperCollider source at the tag `Version-3.14.1`, in
`~/Projects/supercollider`, and sclang and scsynth 3.14.1. In
`SCClassLibrary/Common/Audio`: `SynthDef.sc`, `UGen.sc`,
`BasicOpsUGen.sc`, `Filter.sc`, `FFT.sc`, `FFTUnpacking.sc` and
`BufIO.sc`. In `Common/Core`: `Nil.sc`.

## Facts established on 2026-11-14

The measured facts come from sclang 3.14.1, with probes of one line. The
others come from the sources.

### finishBuild

1. `finishBuild` (`SynthDef.sc:279`) runs `addCopiesIfNeeded`,
   `optimizeGraph`, `collectConstants`, `checkInputs`, `topologicalSort`
   and `indexUGens`, in this order. So the constants follow the optimized
   order of the children, before the sort.

### The copies of PV chains

2. `addCopiesIfNeeded` (`SynthDef.sc:291`, `FFTUnpacking.sc:81`) runs for
   each `PV_ChainUGen`, in the order of `widthFirstUGens`.
   - The readers of a chain are the children that are width-first and not
     a `PV_Copy`, and that hold the chain among their inputs. They come in
     the order of the children.
   - With two readers or more, a slot that holds the chain gets a copy, in
     each reader but the last. A copy is
     `frames = this.fftSize`, `LocalBuf(frames)` and `PV_Copy(this, buf)`.
     It goes right after the chain, so the copy of a later reader comes
     first.
   - `frames` and the `LocalBuf` have no width-first antecedents. The
     `PV_Copy` takes the antecedents of the chain, then its `LocalBuf`.
     Each `LocalBuf` counts one more in the `MaxLocalBufs` of the build.
   - Measured: three readers give two copies. `PV_Mul(c, c)` with an
     `IFFT(c)` gives two copies, one for each slot. A `SpecCentroid` is not
     width-first, so it is no reader.
3. `fftSize` has three owners. `FFT` gives a new `BufFrames.ir(inputs[0])`
   (`FFT.sc:22`), and `PackFFT` gives `inputs[1]` (`FFTUnpacking.sc:31`).
   `PV_ChainUGen` gives the `fftSize` of `inputs[0]`
   (`FFTUnpacking.sc:115`).
4. Measured: a chain that needs a copy fails in sclang in three ways.
   - If the build has no `LocalBuf` yet, `LocalBuf.new1` (`BufIO.sc:150`)
     appends a `MaxLocalBufs`. Then `drop(-3)` drops the wrong UGens.
     `FFT(0, sig)` with two readers gives a definition of four UGens:
     `SinOsc`, `FFT` and two `BufFrames`, with no `IFFT` and no `Out`. With
     a `LocalBuf` elsewhere in the definition, the copy works.
   - A chain of `PackFFT` throws "Message 'widthFirstAntecedents_' not
     understood".
   - A chain of `FFTTrigger` throws "Message 'fftSize' not understood".

### optimizeGraph

5. `optimizeGraph` (`SynthDef.sc:478`) works in three stages.
   - First `initTopoSort` (`SynthDef.sc:501`) gives each child a Set of
     readers. The readers of a UGen read its outputs, through
     `input.source`, or hold it as a width-first antecedent. Then the Set
     becomes an Array, sorted by `synthIndex`.
   - Then `optimizeGraph` visits a copy of the children in order, while a
     rewrite is in progress. A new UGen is not appended during a rewrite.
     `removeUGen` puts nil in the slot of a UGen. `replaceUGen`
     (`SynthDef.sc:447`) puts the new UGen in the old slot, and rewires
     every child.
   - Last, the nils go, and the children get new indices.
6. Each class has one of three behaviors (`UGen.sc:545-610`,
   `BasicOpsUGen.sc:54-309`, `Filter.sc:357`).
   - `UGen` and `DetectSilence` do nothing.
   - `PureUGen`, `PureMultiOutUGen` and `UnaryOpUGen` remove dead code. A
     UGen with no readers removes itself from the readers of each direct
     input, and visits that input again, in the order of the inputs. Then
     it removes itself.
   - `BinaryOpUGen` removes dead code first. Then a `+` tries `Sum3`,
     `Sum4`, `MulAdd` and `a + b.neg`, in this order. A `-` tries
     `a - b.neg`, which becomes a `+` and gets a visit at once.
   - A rewrite fuses only an input with exactly one reader. The result
     takes the readers of the UGen that it replaces.
     `optimizeUpdateDescendants` then updates the readers of the inputs of
     the result.
7. Measured quirks:
   - Dead code stops at an `OutputProxy`. A proxy has no readers, and
     `Nil:remove` does nothing (`Nil.sc:92`). In
     `{ DC.ar(1) * 2; Out.ar(0, SinOsc.ar) }`, the `*` goes, and the `DC`
     stays.
   - A full array of readers loses an update. `Set:asArray.sort` of `n`
     readers gives an Array with room for 2 items at `n = 0`. From 1 on,
     it has room for `max(4, n.next_power_of_two())` items, measured up to
     128. So it is full at 4, 8, 16, 32, 64 and 128 readers. On a full
     Array, `desc.add(r)` returns a new Array, so
     `optimizeUpdateDescendants` loses its update.
   - So `x * 2 + 1` with three more unread readers of `x` gives a
     definition with no UGens, of 43 bytes. Dead code removes `x` while
     the `MulAdd` reads it. With two more readers, the definition is right.
   - The order of the inputs counts. In
     `{ m = x * 2; a = m + 1; u = m * a; Out.ar(0, a) }`, the unread `u`
     makes `a` a `MulAdd`. With `u = a * m`, it does not.
   - The insertion sort of `Sum3:new1` and `Sum4:new1` reverses inputs of
     one rate. `(a + b) - c.neg` gives `Sum3(c, b, a)`. Five terms give a
     `Sum4` and a `+`.
   - A sum at demand rate gets no `Sum3`.
8. Measured: a rewrite whose constructor folds breaks sclang.
   - `x + UnaryOpUGen('neg', 0)` becomes `x - 0`, which folds to `x`.
     `replaceUGen` then writes `SinOsc` and `Out` twice.
   - `2 + UnaryOpUGen('neg', 3)` becomes `2 - 3`, a number. sclang throws
     "Message 'descendants_' not understood".
   - The crate reaches both, because `unary_op` makes a UGen of a number.
9. Measured: an unread `LPF.ar(SinOsc.kr(1))` builds in sclang, because
   `checkInputs` runs after `optimizeGraph`.

## Version 0.7

### The catalogue

- A new enum `Optimize`: `Nothing`, `DeadCode` and `BinaryOp`, and a new
  field `UGenClass::optimize`. `generator/catalog.scd` takes it from the
  owner of `optimizeGraph`, as it takes the rule of `checkInputs`. An
  unknown owner fails the script.
- A new enum `FftSize`: `BufFrames`, `Input` and `Chain`, and a new field
  `UGenClass::fft_size`. The script takes it from the owner of `fftSize`.
  It is `None` for a class without `fftSize`, and an unknown owner fails
  the script.
- The doc of `is_pure` drops its claim about `optimizeGraph`, which is
  false for `DetectSilence`. `optimize` owns that fact.
- `superclasses` tells whether a class is a `MultiOutUGen` or a
  `PV_ChainUGen`.

### The rewrite

A new module `src/optimize.rs` transcribes the methods of sclang of facts
2 to 8. It works on an arena of the nodes of the build:

- The children, with empty slots.
- The width-first antecedents of each node, as a list. A `PV_Copy` takes
  the list of the chain, then its `LocalBuf`.
- The facts of the class of each node, from `catalog::lookup`. An unknown
  class does nothing.
- Arrays of readers with the room of sclang. A replacement shares the
  array of the UGen that it replaces. `add` pushes only while the array has
  room, and `remove` takes the first match. A proxy or a number has no
  array.

It runs in four steps.

1. `add_copies` walks the width-first nodes in order, the appended copies
   included. `frames` comes from `fft_size`, through `Chain` down to a root
   of `BufFrames`. It refuses where fact 4 fails: no `MaxLocalBufs`, a root
   of `Input`, or a root without `fftSize`.
2. `optimize_graph` sets up the arrays of readers as `initTopoSort` does.
   Then it visits the copy of the children, with the recursion of
   `performDeadCodeElimination` in the order of the inputs. It applies the
   other methods of fact 6 as sclang writes them.
3. The constructors of a rewrite fold as sclang folds: `MulAdd.new1`,
   `Sum3.new1`, `Sum4.new1`, the operator `-`, and `BinaryOpUGen.new` of
   `+`. Each gives back a new node, an existing input or a number. Only a
   new node goes on, and the other two refuse (fact 8). The operator
   computes two numbers, which refuses. `BinaryOpUGen.new` makes a UGen of
   them, as sclang does.
4. The empty slots go. A kept node that reads a removed node refuses
   (fact 7). The rewrite gives back the kept nodes in order.

### The builder

`Node` and `Source` become `pub(crate)`. `finish` runs `add_copies` and
`optimize_graph`. Then it writes the count of `MaxLocalBufs`, with the
copies, and runs its constants, its sort and its limits on the new order.

### The error

`BuildError::FinishBuild { ugen, message }` covers each case where the
`finishBuild` of sclang 3.14.1 throws or writes a broken definition. Its
message names the case.

### The generator

- The section 0.7 of `generator/fixtures.scd` writes full builds only.
- A list of refused cases must break in sclang. A case breaks if
  `finishBuild` throws, an `AbstractOut` goes missing, or a UGen comes
  twice. Else the script fails. So the script owns the facts behind each
  refusal.
- In step 4, `buildBoth` and the folder `before_optimize` go. `writePseudo`
  becomes `writeCases`. The check that `finishBuild` leaves a graph of 0.1
  or 0.2 alone goes too. The script then writes every other file again,
  byte for byte.

## Differences from sclang that remain

- The refusals of `BuildError::FinishBuild`, where sclang breaks or throws
  (facts 4, 7 and 8).
- When `ugen` adds a UGen, the crate checks its inputs. So a UGen that
  sclang removes still gets its check (fact 9).

## Tests

- `generator/fixtures.scd` writes three files.
  - `optimize` holds `MulAdd` in four orders and at control rate, and two
    cases that do not fuse. It also holds `Sum3` on the left and on the
    right, `Sum4`, five terms and `a + a`. Last come the rewrites of `neg`,
    and a sum at demand rate.
  - `deadcode` holds an unread chain, `DC.ar(1) * 2` and an unread
    `DetectSilence`. It also holds an unread operator on an argument, and
    the constant of a removed UGen. Last come the pair `m * a` and `a * m`,
    and `x * 2 + 1` with two more readers.
  - `pvcopy` holds three readers, `PV_Mul(c, c)` with `IFFT(c)`, a deeper
    chain and `SpecCentroid`. It also holds `FFT(0, sig)` with a `LocalBuf`
    elsewhere. These cases need no `optimizeGraph`.
- Six refused cases: `FFT(0, sig)` read twice, a chain of `PackFFT` and a
  chain of `FFTTrigger` read twice, `x * 2 + 1` with three more readers,
  `x + neg(0)`, and `2 + neg(3)`. The script checks that each one breaks in
  sclang, and a test of the crate expects `FinishBuild`.
- `tests/common/optimize.rs` builds each graph. `NAMES` gains the three
  files, and `tests/golden.rs` counts 77 files.
- The 8 graphs of `before_optimize` compare with their files in
  `tests/fixtures`.
- A test of the catalogue counts each `Optimize` and each `FftSize`.
- A unit test owns the room of an array of readers.

## Steps

Each step is one commit on `feat/optimize-graph`. Every commit leaves the
eight commands of `CLAUDE.md` green.

1. `docs: the plan of version 0.7`.
2. `test: the golden files of 0.7, written by sclang 3.14.1`. The folder
   `before_optimize` stays.
3. `feat: the copies of PV chains`: `FftSize`, `add_copies`,
   `BuildError::FinishBuild`, the three refusals of PV chains, and the
   parity of `pvcopy`.
4. `feat: optimizeGraph`: `Optimize`, `optimize_graph`, the three other
   refusals, the parity of `optimize`, `deadcode` and the 8 graphs, and the
   removal of `before_optimize`.
5. `docs: the release of version 0.7`.

## sc-app3

After the tag, sc-app3 takes `v0.7.0` on a branch
`build/scsynth-synthdef-0.7.0` from `develop`. The submodule moves, and
`Cargo.lock` changes one line. Core's definitions hold no `+`, and each of
their UGens has a reader. So they keep their 2,301 bytes. A new test of
sc-expression compiles the markup of `optimize_args`, `osc * amp + 0.5`,
against the golden file of the crate. The PR goes to `develop`, and merges
once its CI is green.

## After version 0.7

The work after version 0.7, in the order of its value for sc-app3:

1. `NamedControl`, `Control.names`, `SynthDef.wrap`, `prependArgs`, the
   specs of the metadata, and `LagControl`
2. `Env`, `VarLag`, and the pseudo-UGens that wait for items 1 and 2:
   `EnvGate`, and the `pvcalc` methods of `PV_ChainUGen`
3. sc3-plugins
