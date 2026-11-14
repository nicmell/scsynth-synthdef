# The plan of scsynth-synthdef

This document is the plan of version 0.6. A session starts from it and from
the README, in this folder. The plan owns its decisions and facts until the
code and the tests own them. Then the facts move to the tests, the README
and the CHANGELOG, and this file keeps only what is still open.

## What we build

Version 0.6 takes item 1 of "Next": an array among the elements of an array
argument, as each class expands it. Version 0.5 flattens an array argument
into the inputs, and refuses an array among its elements with
`BuildError::NestedArray`. So
`Out.ar(0, Pan2.ar(SinOsc.ar([440, 660]), [-0.5, 0.5]))` fails, where
sclang makes two `Out`.

- The 40 addable methods with an array argument take an array among its
  elements as sclang does. 34 of them expand it, `SendReply` flops it, and
  `EnvGen` takes each element as one envelope. `IEnvGen` keeps refusing it.
- Each pseudo-UGen with an array argument gets golden cases with a nested
  array.

Then sc-app3 takes the release, so that its markup can wire a nested array
into `Out.ar`.

- The crate stays a library for any Rust user. Nothing of sc-app3 enters it.
- The crate keeps its rules: no I/O, serde as its only optional dependency,
  the MSRV 1.85, the `wasm32-unknown-unknown` build, its lints, and docs on
  every public item.

## Ground rules

- This is a clean implementation from the SuperCollider sources. Do not
  consult earlier compilers: neither `sc-app2/packages/synthdef-compiler`
  nor `sc-crates/crates/scsynthdef-compiler`. No search of sc-app3 reaches
  its submodule `sc-app2`.
- Where sclang writes a definition that scsynth misreads or refuses, the
  crate refuses the input.
- A crate item lands with its first caller, so that no commit warns about
  dead code.
- Never touch the scsynth on UDP 57110.

## Decisions

The owner took these on 2026-11-14.

1. Version 0.6 takes item 1 of "Next". Items 2 to 5 wait.
2. The plan goes to the owner as a plan, with no artifact.
3. An array of specs for `Klank`, `Klang`, `DynKlank` and `DynKlang` stays
   out. sclang tells a spec from an array of specs by its `Ref`, which
   `ArgValue` does not have. A rule by depth clashes with the parts of
   `DynKlank`, which can hold arrays, for several channels.
4. When the CI of the PR is green, the work stops. On the go of the owner,
   the PR merges, and `v0.6.0` becomes an annotated tag on its merge commit.
   Then sc-app3 moves, and its PR merges into `develop` once its CI is
   green.

## Sources of truth

The SuperCollider source at the tag `Version-3.14.1`, in
`~/Projects/supercollider`, and sclang and scsynth 3.14.1. In
`SCClassLibrary/Common/Audio`: `UGen.sc`, `InOut.sc`, `BufIO.sc`,
`DiskIO.sc`, `Demand.sc` and `Osc.sc`. Also `Noise.sc`, `Trig.sc`,
`EnvGen.sc`, `IEnvGen.sc` and `Splay.sc`. In `Common/Collections`:
`Array.sc` and `ArrayedCollection.sc`.

## Facts established on 2026-11-14

The measured facts come from sclang 3.14.1. The others come from the
sources.

### The array arguments of the UGens

1. Only an argument of the class `Array` expands in `multiNewList`
   (`UGen.sc:23-39`). So a `Ref` does not expand.
2. 34 methods hand each element of the array to `multiNewList`, as
   `multiNewList(['audio', bus] ++ channelsArray)` does. They are the
   methods of `Out`, `ReplaceOut`, `OffsetOut`, `LocalOut`, `XOut`,
   `SharedOut`, `BufWr` and `RecordBuf`. Then come those of `DiskOut`,
   `ScopeOut`, `ScopeOut2`, `BelaScopeOut`, `Select`, `TWindex` and
   `Demand`. Last come `ListDUGen` and its subclasses, `Dswitch1` and
   `Dswitch`. So an element that is an array expands the UGen together with
   the other arguments, to any depth. Measured: `Dseq([[1, 2], 3])` makes
   two `Dseq`.
3. `SendReply` flops (`Trig.sc:30-50`). With an array among the values
   (`containsSeqColl`), each element of the values gives the values of one
   instance. Else the values stay whole (`bubble`). The flop goes one
   level, over `trig`, `cmdName`, the values and `replyID`. Measured:
   `[a, [b, c]]` makes two `SendReply`, with one value and with two.
4. If the first element is an array, `EnvGen:convertEnv` makes a `Ref` of
   each element, because `shape` looks only at the first element
   (`EnvGen.sc:72-82`, `ArrayedCollection.sc:337`). Else it makes one `Ref`
   of the whole array. `IEnvGen:convertEnv` makes one `Ref` of any array
   (`IEnvGen.sc:13-16`).
5. `Array:isValidUGenInput` answers true (`Array.sc:217`). So sclang builds
   a UGen with an array among its inputs. It counts the array as one input,
   and writes an input spec for each of its elements, so scsynth misreads
   the file. Measured: `EnvGen.kr([0, [1, 2], -99, -99])`,
   `IEnvGen.kr([[0, 1], 2], 0.5)` and `SendReply.kr(t, '/x', [[[a, b], c]])`
   build such files.
6. Only with a zero at the top level, `replaceZeroesWithSilence` goes into
   the inner arrays, as `lang::replace_zeroes_with_silence` does. Measured:
   `Out.ar(0, [[0, a], b])` fails with "Out input at index 1 ( 0 ) is not
   audio rate", and `Out.ar(0, [[0, a], 0])` builds
   `[SinOsc, DC, Out, DC, Out]`.

### The pseudo-UGens

7. `LocalIn` (`*default`, `InOut.sc:182-193`) and `Splay` (`++ inArray`,
   `Splay.sc:36-41`) hand the elements to `multiNew`, and their bodies
   already expand them. `Mix` and `Splay` got nested golden cases in 0.3.
   Six bodies reach the new behavior through `ugen`: `SelectX`,
   `LinSelectX`, `TChoose`, `TWChoose`, `PingPong` and `MultiTap`. They
   call `Select`, `TWindex`, `BufWr` and `RecordBuf`. Measured: `PingPong`
   with `[[a, b], c]` builds two `BufWr`.

## Version 0.6

### The catalogue

- A new enum `ArrayExpansion`: `Elements` (fact 2), `Flop` (fact 3) and
  `Refs` (fact 4).
- A new field `Wiring::array_expansion: Option<ArrayExpansion>`. It is
  `None` for a method without an array argument, and for `IEnvGen`, whose
  array stays among the inputs.
- A new probe in `generator/catalog.scd`. For each addable method with an
  array argument, it puts `[[e0, e1], e2]` there, then `[e2, [e0, e1]]`. It
  classifies the method by the instances that it makes:
  - Elements: two instances with `[e0, e2]` and `[e1, e2]`, then two with
    `[e2, e0]` and `[e2, e1]`, and three for `[[[e0, e1], e2], e3]`.
  - Flop: two instances with `[e0, e1]` and `[e2]`, then two instances.
  - Refs: two instances with `[e0, e1]` and `[e2]`, then one instance that
    holds the array.
  - None: one instance that holds the array.

  Anything else fails the script. The reading of the sources predicts 34,
  2, 2 and 2.

### The builder

- `ugen` adds the array argument to the expansion, after the silence of its
  zeros:
  - `Elements`: each element is a value that expands, under the name of the
    argument. The leaf rebuilds the array of its instance from them.
  - `Flop` with an array among the elements, and `Refs` with an array as
    the first element: the indices of the elements expand in their place.
    So the expansion wraps them as it wraps the elements, and goes no
    deeper. The leaf takes the element of its index whole.
  - In any other case, the array goes whole to each instance.
- `wire_inputs` keeps refusing an array that stays among the inputs of an
  instance, with `NestedArray` (fact 5).
- `expand` does not change.

### The pseudo-UGens

Fact 7 predicts no change. Step 4 tests each array argument with a nested
array, and fixes a body that differs from sclang.

## Differences from sclang that remain

- An array that stays among the inputs of a UGen (fact 5): sclang writes a
  file that scsynth misreads, and the crate refuses it.
- An array of specs for the four classes of decision 3.

## Tests

- `generator/fixtures.scd` writes five files: `nestedout`, `nestedwire`,
  `nestedreply`, `nestedenv` and `nestedpseudo`. A case of `nestedpseudo`
  that sclang refuses becomes a refusal test of the crate.
- `tests/common/pseudo.rs` builds each graph. `NAMES` grows at each step,
  so `tests/golden.rs`, `tests/parity.rs` and `tests/scsynth.rs` cover it.
- `Out.ar(0, [[0, a], b])` fails the rule of `AbstractOut`, as in sclang.
  `SendReply` with `[[[a, b], c]]`, `EnvGen` with `[0, [1, 2], -99, -99]`
  and `IEnvGen` with `[[0, 1], 2]` give `NestedArray`.
- A test of the catalogue counts each `ArrayExpansion`.

## Steps

Each step is one commit on `feat/nested-arrays`. Every commit leaves the
eight commands of `CLAUDE.md` green.

1. `docs: the plan of version 0.6`.
2. `test: the golden files of 0.6, written by sclang 3.14.1`.
3. `feat: the arrays inside array arguments`.
4. `test: the pseudo-UGens on nested arrays`. A fix of a body makes it a
   `fix:` commit.
5. `docs: the release of version 0.6`.

## sc-app3

After the tag, sc-app3 takes `v0.6.0` on a branch
`build/scsynth-synthdef-0.6.0` from `develop`. The submodule moves, and
`Cargo.lock` changes one line. Core does not change, and its definitions
keep their 2,301 bytes. A test of sc-expression expects the refusal of a
nested array, and loses that case. A new test compiles the markup of
`nestedout_pan2` against the golden file of the crate. The PR goes to
`develop`, and merges once its CI is green.

## After version 0.6

The work after version 0.6, in the order of its value for sc-app3:

1. `optimizeGraph`, and `addCopiesIfNeeded` for PV chains. Each file of
   `tests/fixtures/before_optimize` and the file of its name in
   `tests/fixtures` are a graph before and after `optimizeGraph`.
2. `NamedControl`, `Control.names`, `SynthDef.wrap`, `prependArgs`, the
   specs of the metadata, and `LagControl`
3. `Env`, `VarLag`, and the pseudo-UGens that wait for items 2 and 3:
   `EnvGate`, and the `pvcalc` methods of `PV_ChainUGen`
4. sc3-plugins
