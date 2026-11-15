# The plan of scsynth-synthdef

This document is the plan of version 0.9. A session starts from it and from
the README, in this folder. The plan owns its decisions and facts until the
code and the tests own them. Then the facts move to the tests, the README
and the CHANGELOG, and this file keeps only what is still open.

## What we build

Version 0.9 takes item 1 of "Next": `Env`, `VarLag`, and `EnvGate`, which
waits for `Env`. Today `EnvGen` and `IEnvGen` take only a raw envelope
array. `ugen` refuses `VarLag.ar` and `VarLag.kr`, and `EnvGate` has no
body.

- Each graph with an `Env`, a `VarLag` or an `EnvGate` writes the bytes
  that sclang 3.14.1 writes.
- Where sclang throws, the crate refuses with a `BuildError`.

Then sc-app3 takes the release. sc-app3 wants the envelopes of sc-app2, as
`adsr` and `perc`, in both of its domains: the scalar domain of the vars of
a page, and the graph domain of a synthdef.

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

The owner took these on 2026-11-15.

1. `SynthDefBuilder::env(selector, args)` gives an `Env` value. It takes the
   arguments by name, with the defaults of sclang from the catalogue, as
   `ugen` takes them. `EnvGen` and `IEnvGen` take the `Env` as their
   `envelope`. Getters give both arrays. In the scalar domain, a scratch
   build computes plain numbers.
2. A new `Curve` value holds a shape name, a number, an output, or a curve
   per segment.
3. The scope is the constructors, both arrays, `kr` and `ar`, and `delay`.
   `range`, `exprange` and `curverange` stay out, because they need
   `UGen:linlin` and its family. So do the methods that only make sense in
   the language, such as `at`, `plot` and `test`.
4. The crate copies every quirk of sclang 3.14.1. It refuses what sclang
   throws on, with a `BuildError`.
5. sc-app3 takes 0.9 with the move of its submodule and one byte test,
   through `<sc-ugen type="EnvGate">` and `<sc-ugen type="VarLag">`. The
   functions `adsr` and `perc` get a plan of their own in sc-app3.
6. When the CI of the PR is green, the work stops. On the go of the owner,
   the PR merges, and `v0.9.0` becomes an annotated tag on its merge commit.
   Then sc-app3 moves, and its PR merges into `develop` once its CI is
   green.

## Sources of truth

The SuperCollider source at the tag `Version-3.14.1`, in
`~/Projects/supercollider`, and sclang and scsynth 3.14.1. In
`SCClassLibrary/Common/Audio`: `Env.sc`, `EnvGen.sc`, `IEnvGen.sc` and
`Filter.sc`. In `Common/Control`: `GraphBuilder.sc`.

## Facts established on 2026-11-15

The measured facts come from sclang 3.14.1, with probes of one line. The
others come from the sources.

### The arrays of an envelope

1. `asArray` is `[level0, segments, releaseNode, loopNode]`, then four
   numbers per segment: the level, the time, the shape and the curve value.
   A missing node is -99. Then `flop` makes one array per channel. With one
   channel, `asArray` drops the outer array, and `asMultichannelArray`
   keeps it.
   - The shapes are step 0, lin 1, exp 2, sin 3, wel 4, sqr 6, cub 7 and
     hold 8. `linear`, `exponential`, `sine`, `welch`, `squared` and
     `cubed` are other names of the same shapes. A number or an output is
     the shape 5, and its value is the curve value.
   - An unknown name throws "Env shape not defined."
   - Segment `i` takes the curve `curves.asArray.wrapAt(i)`. An array inside
     a curve makes channels.
2. `asArrayForInterpolation`, the form of `IEnvGen`, is `[offset, level0,
   segments, times.sum]`, then per segment the time, the shape, the curve
   value and the level. It keeps its outer array, even for one channel.
   `times.sum` starts from 0, so the first `0 + t` folds. Each further time
   that is an output adds a `+`.
3. `EnvGen` takes an `Env` through `asMultichannelArray`, as a reference per
   channel. The `Refs` expansion of 0.6 matches it, given `asArray`.
   `IEnvGen` takes an `Env` through `asArrayForInterpolation`, as a
   reference per channel. But it takes a raw array as one reference. So
   only an `Env` gives a multichannel `IEnvGen`.
   - Measured: `IEnvGen.kr(Env([0, [1, 2], 0], [1, 2]), [0.25, 0.5,
     0.75])` gives three `IEnvGen`s, with the channels 0, 1 and 0.

### The constructors

4. The constructors do their math at the time of the `Env`.
   - `new` has the defaults `#[0,1,0]`, `#[1,1]`, `\lin`, nil, nil and 0.
     It wraps the times to `levels.size - 1`. No levels throws "Primitive
     '_ArrayExtendWrap' failed", and one level gives no segment.
   - `triangle` and `sine` halve their duration. If the levels and the
     times of `step` differ in size, `step` throws.
   - `adsr` and `dadsr` compute `[..., peakLevel * sustainLevel, ...] +
     bias`. Measured: `Env.adsr(peakLevel: p, sustainLevel: s)` adds
     `p * s` before the gate control of the same `EnvGen.kr` call. So the
     math cannot wait for the `EnvGen`. A bias that is an output adds a `+`
     per level, and `0 + b` folds.
   - `cutoff` computes `shapeNumber(curve)` at the time of the `Env`, so an
     unknown name throws there. For `\exp`, its last level is -100.dbamp,
     whose f64 bits are those of `10 ** (-100 * 0.05)` (measured).
   - `xyc` takes triplets of a time, a level and a curve. If all times are
     numbers, it sorts the triplets by time. It differentiates the times,
     and the first time becomes the offset. The curve of the last triplet
     goes unused. `pairs` adds its curve to each pair, and a nil curve gives
     `\lin`.
   - `circle` adds `Impulse.kr(0)`, `Delay1.kr` and `Latch.kr` at the time
     of the `Env`. Its first time is `first0Then1` times the last time, a `*`
     unless that time is 1. It also adds an `inf` time, a release node and
     the loop node 0.
5. `delay(t)` repeats the first level, puts `t` before the times and `\lin`
   before an array of curves, and adds 1 to each node. `kr` and
   `ar(doneAction, gate, timeScale, levelScale, levelBias)` are `EnvGen.kr`
   and `EnvGen.ar` of the `Env`, with the arguments in another order.

### VarLag and EnvGate

6. `VarLag.ar` and `VarLag.kr(in, time, curvature, warp = 5, start, mul,
   add)`:
   - If `in` is at scalar rate, or `time` is 0, the method gives back
     `in.madd(mul, add)` before the expansion, as the Lag family does.
     Measured: `VarLag.kr(3, 0.2, mul: 2, add: 1)` gives 7.
   - Else `multiNew` expands `in`, `time`, `curvature`, `warp` and `start`.
     Measured: `warp: [5, 1]` gives one instance of each kind below. Each
     instance takes `start ? in` and `curve = Env.shapeNames[warp] ? warp`.
   - A curve of 1 makes a real `VarLag` of `in`, `time` and `start`, with
     the rule of `Filter`. So `\lin`, `\linear` and the number 1 do.
     Measured: `VarLag.ar(\x.kr(1), 0.2, warp: \lin)` throws "first input
     is not audio rate".
   - Any other curve makes `Changed(in) + Impulse(0)`, plus
     `Changed.kr(time)` for a time that is not at scalar rate. Then it
     makes the `EnvGen` of `Env([start, in], [time], warp).asArray`, with
     the curve at index 6 and the curvature at index 7. An unknown name
     throws "Env shape not defined". Measured: a warp that is an output
     builds, because `!=` of sclang on a UGen is identity, and the output
     goes to index 6.
   - `madd` applies `mul` and `add` to the result.
7. `EnvGate.new(i_level = 1, gate, fadeTime, doneAction = 2, curve = \sin)`
   makes `NamedControl.kr(\gate, 1.0)` and then `NamedControl.kr(\fadeTime,
   0.02)` for each argument that is nil. Then it makes `EnvGen.kr(Env([i_level,
   1, 0], [1, 1], curve, 1), gate, 1, 0, fadeTime, doneAction)`. Two
   `EnvGate`s share the controls.

## Version 0.9

### The catalogue

- A new field `UGenClass::convert_env`, `Array` or `Interpolation`, from the
  class that owns `convertEnv`. An unknown owner fails the script.
- A new table `env_methods` holds the class methods of `Env` that make an
  envelope, with their arguments. Each other class method of `Env` needs a
  reason in a table, or the script fails. `catalog::env_methods` and
  `catalog::env_method` give them.
- `ArgKind::Curve` marks each `curve`, and the `warp` of `VarLag`.
  `ArgDefault::Floats` holds the array defaults of `new` and `step`. A
  symbol default binds on a `Curve` argument too.
- `VarLag` and `EnvGate` become classes of bodies.

### The envelope

- `Env` holds the levels, the times, the curves, the release node, the loop
  node and the offset, as the constructor of sclang leaves them. `Curve` is
  a shape by name, a value, or an array of curves.
- `Env::as_array`, `Env::as_multichannel_array` and `Env::delay` do no math,
  so they need no build. They give `BuildError::BadEnv` for an unknown
  shape.
- `SynthDefBuilder::env` runs a body per constructor of fact 4, with the
  arithmetic of the pseudo-UGens. `circle` adds its UGens with `ugen`.
- `SynthDefBuilder::env_array_for_interpolation` gives the array of fact 2,
  and adds the `+` of `times.sum`.
- In `ugen`, an `Env` as the `envelope` of a class with `convert_env`
  converts before the expansion. For `EnvGen`, it becomes `as_array`, which
  the `Refs` expansion takes. For `IEnvGen`, it becomes the channels of
  fact 2. One channel goes whole, and several expand with the index, as
  fact 3 says. An `Env` anywhere else gives `WrongKind`.

### VarLag and EnvGate

- The bodies of `VarLag.ar` and `VarLag.kr` copy fact 6. A warp by name
  stays out of the expansion, and a warp that is an input expands.
- The body of `EnvGate.new` copies fact 7, with a helper of the body of
  `NamedControl`.

### The values and the error

- `ArgValue::Env` and `ArgValue::Curve`. A `Curve` argument also takes a
  text as a shape, and an input as values.
- `BuildError::BadEnv { method, message }` refuses no levels, an unknown
  shape, levels and times of `step` that differ in size, and an unknown
  `warp` of `VarLag`. Its message names the case.

## Differences from sclang that remain

- A release or loop node that is an output: the crate takes numbers, so
  `delay` needs no math.
- A name as the curve of a triplet of `xyc`: use `pairs` with a curve per
  segment.
- The scope of decision 3.

## Tests

- `generator/fixtures.scd` writes four files.
  - `env` holds `EnvGen` with each constructor, each shape, curves per
    segment, channels, nodes, outputs as levels, curves and bias, `circle`,
    `delay`, one level, and `Env.kr` and `Env.ar`.
  - `ienv` holds `IEnvGen` with one channel, an offset, times that are
    outputs, two channels with one index and with three, `madd`, and `xyc`.
  - `varlag` holds both rates at the default warp, and both kinds of curve
    1. It also holds `\exp` with a time that is an output, a number curve,
    a `start`, and a warp that is an output. Last come arrays of `in` and
    of `warp`, `madd`, and the two passthroughs.
  - `envgate` holds the defaults, all arguments, two `EnvGate`s, a level
    that is an output, and `envgate_markup`, the graph of the test of
    sc-app3.
- Each refused case must break in sclang, and a test of the crate expects
  its error. The cases are no levels, an unknown shape at the conversion
  and at `cutoff`, and `step` with sizes that differ. Then an unknown
  `warp`, and `VarLag.ar` at `\lin` on an input at control rate.
- `tests/common/envelopes.rs` builds each graph, in the order of sclang.
  `NAMES` gains the four files, and `tests/golden.rs` counts 85 files.
- A test of the catalogue counts `convert_env` and `env_methods`.
  `tests/readme.rs` counts 19 refused methods.
- `tests/scsynth.rs` loads the new graphs.

## Steps

Each step is one commit on `feat/envelopes`. Every commit leaves the eight
commands of `CLAUDE.md` green.

1. `docs: the plan of version 0.9`.
2. `test: the golden files of 0.9, written by sclang 3.14.1`.
3. `feat: Env`: the catalogue, `Env`, `Curve`, the values, `BadEnv`, `env`,
   both arrays, `delay`, the conversion for `EnvGen` and `IEnvGen`, and the
   parity of `env` and `ienv`.
4. `feat: VarLag`: the body, and the parity of `varlag`.
5. `feat: EnvGate`: the body, and the parity of `envgate`.
6. `docs: the release of version 0.9`.

## sc-app3

After the tag, sc-app3 takes `v0.9.0` on a branch
`build/scsynth-synthdef-0.9.0` from `origin/develop`. The submodule moves,
and `Cargo.lock` changes one line. Core's definitions keep their 2,301
bytes. A new test of sc-expression compiles the markup of
`envgate_markup` against the golden file of the crate. The PR goes to
`develop`, and merges once its CI is green.

## After version 0.9

The work after version 0.9, in the order of its value for sc-app3:

1. The `pvcalc` methods of `PV_ChainUGen`, which take a function and need
   `PackFFT`
2. sc3-plugins
