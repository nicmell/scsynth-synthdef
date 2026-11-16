# The plan of scsynth-synthdef

This document is the plan of version 0.12. A session starts from it and
from the README, in this folder. The plan owns its decisions and facts
until the code and the tests own them. Then the facts move to the tests,
the README and the CHANGELOG, and this file keeps only what is still open.

## What we build

Version 0.12 takes item 1 of "Next": the instance methods of `UGen` that
build graphs, such as `range`, `linlin` and `lag`. With them come `range`,
`exprange` and `curverange` of `Env`. They are the most common idioms of
sclang, as `SinOsc.kr(1).range(200, 800)`. Today a user of
the crate writes the `MulAdd` by hand, and its constants can fold otherwise
than sclang folds them.

- Each graph with one of these methods writes the bytes that sclang 3.14.1
  writes.
- Where sclang throws, or writes a definition that is broken, the crate
  refuses with a `BuildError`.

Then sc-app3 takes the release. Its markup reaches classes, not methods.
So the expression functions of the new methods, such as `range(x, lo,
hi)`, get a plan of their own in sc-app3.

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

1. `SynthDefBuilder::send(receiver, selector, args)` sends a message as
   sclang sends it. An output gets the method that its class answers. An
   array hands the message to each element, as `multiChannelPerform` does.
   If `Float` has the method, a number gets it.
2. A new table `ugen_methods` of the catalogue holds the instance methods
   of `UGen` that build graphs, with their arguments and defaults. Every
   other instance method of `UGen` needs a reason in a table, or the
   generator fails.
3. `UGenClass::signal_range` comes from sclang, through the generator.
4. The crate copies every quirk of sclang 3.14.1. It refuses what sclang
   throws on, with a `BuildError`.
5. `SynthDefBuilder::send_env(env, selector, args)` takes `range`,
   `exprange` and `curverange` of `Env`.
6. sc-app3 takes 0.12 with the move of its submodule. Its expression
   functions come later, in a plan of sc-app3.
7. When the CI of the PR is green, the work stops. On the go of the owner,
   the PR merges, and `v0.12.0` becomes an annotated tag on its merge
   commit. Then sc-app3 moves, and its PR merges into `develop` once its CI
   is green.

## Sources of truth

The SuperCollider source at the tag `Version-3.14.1`, in
`~/Projects/supercollider`, and sclang and scsynth 3.14.1. These files of
`SCClassLibrary/Common` hold the methods:

- `Audio/UGen.sc`, `Audio/Demand.sc` (`DUGen`) and `Audio/Osc.sc`
  (`LFGauss`).
- `Audio/Line.sc` (`LinLin`, `LinExp`) and `Audio/Env.sc`.
- `Math/SimpleNumber.sc`, `Math/Float.sc` and `Core/Object.sc`.
- `Collections/SequenceableCollection.sc` and `Collections/Array.sc`.

## Facts established on 2026-11-16

The measured facts come from sclang 3.14.1, with probes of one line on
stdin.

### The methods

1. `UGen` has over 100 instance methods, and the 42 below build graphs.
   The others are steps of the build, such as `init` and `writeDef`. Some
   give objects of the language, such as `@` and `scope`. `madd` is
   `SynthDefBuilder::madd`.
   - `range`, `exprange`, `curverange`, `unipolar` and `bipolar`.
   - `linlin`, `linexp`, `explin`, `expexp`, `lincurve`, `curvelin`,
     `bilin` and `prune`.
   - `clip`, `fold` and `wrap`.
   - `lag`, `lag2`, `lag3`, `lagud`, `lag2ud`, `lag3ud`, `varlag` and
     `slew`.
   - `blend`, `minNyquist`, `degrad`, `raddeg`, `moddif`, `sanitize`,
     `poll`, `dpoll` and `checkBadValues`.
   - `snap`, `softRound`, `binaryValue`, `isPositive`, `isNegative`,
     `isStrictlyPositive`, `|==|`, `degreeToKey` and `if`.
2. Two subclasses override some of them. `DUGen`, the superclass of `Dseq`
   and the other demand UGens, has its own `linlin`, `linexp`, `explin` and
   `expexp`, which compute with operators. Its `expexp` passes `clip /
   inMin` as the type of `prune`, and nests its `log` otherwise than
   `UGen:expexp`. `LFGauss` has its own `range`: `this.linlin(minval, 1,
   lo, hi)`, where `minval` is `exp(1.0 / (-2.0 * squared(width)))` of its
   input `width`.

### The ranges

3. `range(lo, hi)` is `MulAdd(this, mul, add)`. For a `signalRange` of
   `\bipolar`, `mul` is `(hi - lo) * 0.5` and `add` is `mul + lo`. For
   `\unipolar`, they are `hi - lo` and `lo`.
   - `exprange` is `linexp(-1, 1, lo, hi, nil)`, or `linexp(0, 1, ...)` for
     `\unipolar`. `curverange` is `lincurve` in the same way.
   - `unipolar(mul)` is `range(0, mul)`, and `bipolar(mul)` is
     `range(mul.neg, mul)`.
4. `signalRange` is `\bipolar`, from `UGen`. Thirteen classes return
   `\unipolar`, such as `LFPulse`, `Impulse` and `Dust`. Their subclasses
   inherit it, such as `MouseY` and `TDelay`.
   - `newCopyArgs.signalRange` gives it for each class. `basicNew` is not
     understood.
   - An `OutputProxy` keeps `\bipolar`: an output of `Pan2.kr` takes the
     `MulAdd` of `\bipolar`.

### The maps

5. `linlin` is `LinLin.ar`, for a receiver at audio rate, or `LinLin.kr`
   of `this.prune(inMin, inMax, clip)`. `linexp` is `LinExp.multiNew` at
   the rate of the receiver, of the same.
   - `explin` and `expexp` compute with `log` and `pow`.
   - `lincurve` and `curvelin` give `linlin` for a number curve under
     0.125. For a curve at scalar rate, they give the arithmetic. Else
     they give `Select` of `abs(curve) >= 0.125`, by the
     `methodSelectorForRate` of the receiver.
   - `bilin` is `Select` of two `linlin` by `this < inCenter`.
   - `prune` is `clip` for `\minmax`, `max` for `\min`, `min` for `\max`,
     and the receiver for any other type.
6. `clip`, `fold` and `wrap` call the class by its `methodSelectorForRate`:
   `ar`, `kr`, or `ir` at scalar rate. At demand rate, `clip` is `max(lo,
   min(hi, this))`, and `fold` and `wrap` throw `NotYetImplementedError`.

### The other methods

7. The lag family calls `multiNew` at the rate of the receiver, so it
   skips the passthrough of the rate methods: `x.lag(0)` adds a `Lag`. A
   `t2` gives the `UD` classes. `varlag` is the `VarLag` of 0.9 without its
   passthrough, and `slew` is `Slew.multiNew`. A receiver at scalar or
   demand rate gives a `Lag` at that rate.
8. The rest:
   - If the receiver or `that` runs at audio rate, `blend` is `XFade2.ar`.
     Else it is `LinXFade2` by the `methodSelectorForRate` of `LinXFade2`,
     with `blendFrac.linlin(0, 1, -1, 1)`.
   - `minNyquist` is `min(this, SampleRate.ir * 0.5)`. `degrad` and
     `raddeg` multiply by a constant.
   - `moddif` is `ModDif.multiNew`, and `sanitize` is `Sanitize` by the
     `methodSelectorForRate` of the receiver.
   - `poll` is `Poll(trig, this, label, trigid)`, and `dpoll` is
     `Dpoll(this, label, run, trigid)`.
   - `checkBadValues` adds its UGen, and gives back the receiver.
   - `snap` and `softRound` are `Select.multiNew` of `round`, at the rate of
     the receiver. `binaryValue` is `sign.max(0)`. `isPositive`,
     `isNegative` and `isStrictlyPositive` compare with 0, and `|==|` is
     `==`.
   - `degreeToKey` is always `DegreeToKey.kr`, and `if(t, f)` is
     `(this * (t - f)) + f`.
9. `methodSelectorForRate` of an instance gives `new` at demand rate, and
   at scalar rate `ir` or `new`, by whether the class of the receiver
   answers `ir`. `Select`, `Sanitize`, `LinXFade2` and `CheckBadValues`
   answer neither as a UGen. So sclang writes a broken definition, without
   its `Out`, for these at scalar or demand rate:
   - `sanitize`, and `blend` at scalar rate
   - `lincurve` and `curvelin` with a curve that is an output
   - `bilin`

   `checkBadValues` at these rates adds nothing, and gives back the
   receiver. The definition stays whole.

### The receivers

10. A number answers 31 of the 42 methods, by the method of `Float`,
    `SimpleNumber` or `Object`.
    - The lag family gives the number. `linlin`, `linexp`, `explin`,
      `expexp`, `lincurve`, `curvelin` and `bilin` compute it, after
      `if` tests of the clip. `0.25.linlin(0, 1, 200, 800)` is 350.0.
    - `clip`, `fold` and `wrap` are primitives of `Float`, which fail on an
      argument that is not a number.
    - `isPositive`, `isNegative`, `isStrictlyPositive`, and `|==|` of two
      numbers, give a Boolean, which `Out` refuses as an input.
    - `0.5 |==| x` is `x == 0.5` for a UGen `x`.
    - A number does not understand `range`, `exprange`, `curverange`,
      `unipolar`, `bipolar`, `prune`, `minNyquist`, `sanitize`, `dpoll`,
      `checkBadValues` or `if`.
11. An array answers 38 of the 42 methods.
    - Most go through `multiChannelPerform`: a `flop` of the receiver and
      the arguments, then the method for each element.
      `[SinOsc.kr, LFPulse.kr].range(0, [1, 2])` gives a `MulAdd`, and a
      `*` for `LFPulse`.
    - `degrad`, `raddeg`, `isPositive`, `isNegative` and
      `isStrictlyPositive` send the message to each element. So does
      `degreeToKey`, with the same arguments.
    - `minNyquist` is `min(this, SampleRate.ir * 0.5)`, with one
      `SampleRate`.
    - If `label` is nil, `poll` and `dpoll` label each element "UGen Array
      [i]".
    - An array does not understand `snap`, `softRound`, `binaryValue` or
      `if`.

### `Env`

12. `range`, `exprange` and `curverange` copy the envelope with its levels
    through `linlin`, `linexp` or `lincurve`, from `levels.minItem` to
    `levels.maxItem`. An output among the levels throws "Non Boolean in
    test."

## Version 0.12

### `send`

- `SynthDefBuilder::send(receiver, selector, args)` binds the arguments by
  name, against the method of `UGen` in the table `ugen_methods`, with its
  defaults. Then it passes them all, in their order, to the method that
  the receiver answers. So a call is the sclang call that passes every
  argument by position.
- An output takes the body of its class: `UGen`, or the override of fact
  2. A body adds its UGens through `ugen`, `madd` and the operators of
  `lang`, in the order of sclang.
- An array takes the dispatch of fact 11, and a number the method of fact
  10, through the arithmetic of `lang`.
- `BuildError::BadMethod { selector, reason }` refuses what sclang throws
  on, and the broken definitions of fact 9.
- A call that fails leaves the build as it was.

### The catalogue

- `generator/catalog.scd` gets a table `ugenMethods` of the 42 methods, and
  a table `notUgenMethods` of the others, each with a reason. A method of
  `UGen` that no table names fails the script.
- For each method, the generator writes the class whose method a number
  and an array answer, and the subclasses of `UGen` that override it.
- `UGenClass::signal_range: SignalRange`, `Bipolar` or `Unipolar`, comes
  from `newCopyArgs.signalRange`. Any other value fails the script.
- `catalog::ugen_method` and `catalog::ugen_methods` give the table, and
  `tests/render.rs` renders it.

### `Env`

- `SynthDefBuilder::send_env(env, selector, args)` takes `range`,
  `exprange` and `curverange`, with their defaults from a table of the
  catalogue.
- An output among the levels gives `BuildError::BadEnv`, as sclang throws
  there.

## Tests

- `generator/fixtures.scd` writes six files:
  - `ugenranges`: facts 3 and 4, with each kind of receiver: a bipolar
    class, a unipolar class, an inherited one, an `OutputProxy`, an
    operator, and `LFGauss`.
  - `ugenmaps`: facts 5 and 6, with each type of clip, each rate, and the
    overrides of `DUGen`.
  - `ugenlags`: fact 7, `lag(0)` included.
  - `ugenmisc`: facts 8 and 9.
  - `ugenreceivers`: facts 10 and 11: arrays with array arguments, and
    numbers whose results land as constants.
  - `envranges`: fact 12.
- Each refused case must break in sclang, or be broken by fact 9, and a
  test of the crate expects its error.
- `tests/golden.rs` counts 95 files. `tests/fixtures/wire_buffers.json`
  gains the six files in step 2, because the test of the counts reads
  every golden file. `NAMES` gains each file in its step.
- A test of the catalogue owns the 42 methods, the classes that numbers
  and arrays answer, the overrides, and the unipolar classes.
- `tests/scsynth.rs` loads the new graphs.

## Steps

Each step is one commit on `feat/ugen-methods`. Every commit leaves the
eight commands of `CLAUDE.md` green.

1. `docs: the plan of version 0.12`.
2. `test: the golden files of 0.12, written by sclang 3.14.1`.
3. `feat: send, with the ranges and the maps`: the tables,
   `signal_range`, `send` for outputs, arrays and numbers, `BadMethod`, and
   facts 3 to 6. Then the parity of `ugenranges`, `ugenmaps` and
   `ugenreceivers`.
4. `feat: the other methods of UGen`: facts 7 to 9, and the parity of
   `ugenlags` and `ugenmisc`.
5. `feat: the ranges of Env`: `send_env`, its table, and the parity of
   `envranges`.
6. `docs: the release of version 0.12`.

## sc-app3

After the tag, sc-app3 takes `v0.12.0` on a branch
`build/scsynth-synthdef-0.12.0` from `origin/develop`, in a worktree of
its own. The submodule moves, and `Cargo.lock` changes one line. Core's
definitions keep their bytes. The PR goes to `develop`, and merges once its
CI is green.

## After version 0.12

The work after version 0.12, in the order of its value for sc-app3:

1. sc3-plugins
