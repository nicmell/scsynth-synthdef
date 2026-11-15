# The plan of scsynth-synthdef

This document is the plan of version 0.8. A session starts from it and from
the README, in this folder. The plan owns its decisions and facts until the
code and the tests own them. Then the facts move to the tests, the README
and the CHANGELOG, and this file keeps only what is still open.

## What we build

Version 0.8 takes item 1 of "Next": `NamedControl`, `Control.names`,
`SynthDef.wrap`, `prependArgs`, the specs of the metadata, and
`LagControl`. These are the controls that a graph function makes after its
arguments. Today the crate builds only the controls of the arguments, in
`SynthDefBuilder::new`. `ugen` refuses `Control.kr`, `Control.ir`,
`TrigControl.kr`, `TrigControl.ir`, `AudioControl.ar`, `LagControl.kr` and
`LagControl.ar`, because they take their special index from the build.
`NamedControl` has no body.

- Each graph of these features writes the bytes that sclang 3.14.1 writes.
- Where sclang throws or writes a broken definition, the crate refuses with
  a `BuildError`.

Then sc-app3 takes the release. sc-app3 keeps `<sc-control>` for the named
controls, and its list "Later" waits for `NamedControl` in the crate. With
0.8, `<sc-ugen type="NamedControl">` compiles with no change in
sc-expression.

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

1. `NamedControl` goes through `ugen("NamedControl", "kr", ...)`, as the
   other pseudo-UGens do.
2. The crate copies every quirk of sclang 3.14.1. It refuses what sclang
   throws on, with a `BuildError`.
3. Two entries of one control name: the crate writes both, as sclang does.
   A measure states which entry scsynth sets.
4. The specs give defaults only. For an argument, the caller passes the
   default of the spec, so `Arg` gets no new API. The view of `SynthDesc`
   stays out.
5. sc-app3 takes 0.8 with the move of its submodule and one byte test.
   `<sc-control>` gets a plan of its own in sc-app3, later.
6. When the CI of the PR is green, the work stops. On the go of the owner,
   the PR merges, and `v0.8.0` becomes an annotated tag on its merge commit.
   Then sc-app3 moves, and its PR merges into `develop` once its CI is
   green.

## Sources of truth

The SuperCollider source at the tag `Version-3.14.1`, in
`~/Projects/supercollider`, and sclang and scsynth 3.14.1. In
`SCClassLibrary/Common/Audio`: `SynthDef.sc` and `InOut.sc`. In
`Common/Control`: `GraphBuilder.sc`, which holds `NamedControl`, and
`Spec.sc`. In `Common/Core`: `Symbol.sc`. In `server/scsynth`:
`HashTable.h`.

## Facts established on 2026-11-15

The measured facts come from sclang 3.14.1, with about 90 probe cases. The
others come from the sources.

### NamedControl

1. `NamedControl` makes one control UGen per name, at its first use, after
   the controls of the arguments. Later uses of the name reuse it, also
   inside `SynthDef.wrap`. Each name has a control UGen of its own:
   `\a.ir(1)`, `\b.tr(2)`, `\c.ar(3)` and `\d.kr(4)` give `Control.ir`,
   `TrigControl.kr`, `AudioControl.ar` and `Control.kr`, with the special
   indices 0 to 3.
2. A name whose second byte is `_` has a prefix: its first byte. The first
   match gives the rate.
   - Audio: the prefix `a`, or the rate audio.
   - Trigger: the prefix `t`, or the rate trigger.
   - Scalar: the prefix `i`, or the rate scalar.
   - Else control.
   - So `\i_x.ar` is audio, `\t_y.ir` is a trigger, `\i_z.tr` is a trigger
     and `\a_w.ir` is audio. The arguments use another order: scalar,
     trigger, audio, then control.
   - `NamedControl.new` takes `\audio`, `\trigger` or `\scalar`. Any other
     rate, or nil, gives control.
3. The lags.
   - Each 0 among the lags becomes nil. A lag of 0 alone means no lag.
   - If `fixedLag` is true, or if the rate of the lags is scalar, the lags
     are fixed. A number has the rate scalar. An array has the least rate by
     name of its elements, where nil counts as scalar. `[nil]` has no rate.
   - Fixed lags on a name without a prefix make a `LagControl.kr`, or a
     `LagControl.ar` at audio rate. So `\freq.kr(440, 0.1)` is a
     `LagControl`. `\a.kr(1, \lt.kr(0.2), true)` is a `LagControl` with an
     output as its lag input.
   - Other lags add a `Lag` at each use, at the rate of the control, with
     no passthrough, because `UGen:lag` calls `Lag.multiNew`.
     `NamedControl.ir(\a, 1, \lt.kr(0.2))` gives a `Lag` at scalar rate.
   - The lags expand over the channels. `\x.kr([1, 2], \l.kr)` gives two
     `Lag`s, and `\x.kr(1, [l1, l2])` gives two `Lag`s of one control.
   - A nil inside an array of lags throws. sclang says "LagControl arg:
     'lags' has bad input: nil" or "Lag arg: 'lagTime' has bad input: nil".
4. The order of the uses counts. A fixed lag at the first use gives a
   `LagControl`, and later uses read it, with or without a lag. A first use
   without a lag, then a number lag, gives a `Control` and a `Lag`.
5. Quirks.
   - A prefix skips the `LagControl`. Then the fixed lag returns the bare
     control. So `\x_freq.kr(440, 0.1)` and `\a_.kr(1, 0.1)` drop their
     lag.
   - `NamedControl.ir(\a, 1, 0.1)` and `NamedControl.tr(\a, 1, 0.1)` give a
     `LagControl` at control rate.
6. The values.
   - `\x.kr([[1, 2], [3, 4]])` gives one `Control` of 4 values, and the
     result takes the shape of the values.
   - One value in an array gives one output. No value gives 0.0.
   - An empty array throws "numChannels must be a nonzero positive
     integer". If an output is among the values, the write of the file
     throws.
7. A second use with other default values, another rate, or other fixed
   lags throws. A spec without values gives its default first. So
   `\freq.kr(spec: \freq)`, then `\freq.kr(spec: \amp)`, throws.

### The specs

8. `Spec.specs` holds 23 `ControlSpec`s, with the warps linear, exp, amp
   and db. The spec `db` has the default -inf, and the spec `phase` has the
   maximum 2π.
   - A `NamedControl` with a spec and no values takes the default of the
     spec. An unknown spec name throws "Message 'default' not understood".
   - With `metadata[\specs]`, an argument without a default takes the
     default of its spec, else the default of the spec named like the
     argument, else 0. Without specs, it takes 0.0.

### The names

9. An argument and a `NamedControl` of one name write two entries of that
   name. So do a wrapped argument with the name of an outer argument, and
   `Control.names` with a `NamedControl`.
   - The variants of sclang take the last entry of a name. Measured:
     `[freq: 330]` sets the `NamedControl`.
   - scsynth keeps the first entry of a name in its table of parameters,
     because `HashTable::Add` (`HashTable.h:105`) refuses the second.
10. `Control.names` adds names with the index `controlIndex + i`, and with
    no default.
    - If the last name of the current function has no default,
      `Control.kr` and `TrigControl.kr` give it their values.
      `AudioControl` and `LagControl` give none.
    - So `Control.names([\a, \b]).kr([1, 2])` leaves `a` without a default,
      and gives `b` the default `[1, 2]`.
    - Fewer names than values leave values without a name. More names point
      past the parameters, and sclang writes them.
    - `Control.names([\a, \b])`, then `SynthDef.wrap({ |c = 7| c })`, then
      `Control.kr([1, 2])` give `a` and `c` the index 0.

### The control UGens

11. `Control.kr([[1, 2], 3])` expands into two `Control`s, of `[1, 3]` and
    `[2, 3]`. `TrigControl.ir(1)` is a `TrigControl` at scalar rate.
    - `LagControl.kr` repeats a number lag. It cuts the values and the lags
      into groups of 16, and each group expands as `multiNewList` does. It
      always gives an array.
    - If the values and the lags differ in size, `LagControl.kr` posts an
      error and gives nil. Then its reader throws, and an unread one adds
      nothing. `LagControl.kr([], [])` adds nothing.
    - `LagControl.ar` is `AudioControl.ar(values).lag(lags)`, with no check
      of the sizes.

### SynthDef.wrap and prependArgs

12. `SynthDef.wrap` adds the controls of the arguments of its function, by
    group of rate, after the controls so far. The names come in the order
    of the arguments, and the indices in the order of the groups.
    - It sets `controlNames` to nil around the function. Only `Control.init`
      reads that list.
    - `prependArgs` give their values to the first arguments, which get no
      controls.

### Variants

13. A variant on a name without a default writes the count of the variants
    and no variant, so the file is broken. A variant past the parameters
    throws "Index out of range".

## Version 0.8

### The catalogue

- The census of `generator/catalog.scd` gains `AudioControl`, `Control`,
  `LagControl`, `NamedControl` and `TrigControl` as classes of bodies.
  `NamedControl` leaves the classes left out, and `LagControl` the classes
  that are no pseudo-UGens. The reason of `EnvGate` names the item of
  "Left out" that it waits for.
- If sclang refuses a method of a class of bodies as not valid for its
  subclass, the method stays refused, with its reason. So `LagControl.ir`
  stays in the list of methods that `ugen` refuses.
- The script also looks for `tr`, so `NamedControl` lists `ar`, `kr`, `ir`,
  `tr` and `new`. `name`, `spec` and `rate` take the kind symbol, `values`
  the kind array, and `fixedLag` the kind bool.
- A new table `specs` holds each key of `Spec.specs`, sorted, with its
  minval, maxval, warp, step, default and units. The script fails on a spec
  that is not a `ControlSpec`, and on a warp that is not a symbol.
- In Rust: `ControlSpec`, `catalog::spec(name)` and `catalog::specs()`.

### The control UGens and Control.names

- The builder keeps the size of the default of each name, or none for a
  name of `Control.names`. It also keeps the last name of the current
  function, which `Control.init` reads.
- `SynthDefBuilder::control_names` adds names, as `Control.names` does.
- The bodies of `Control.kr`, `Control.ir`, `TrigControl.kr`,
  `TrigControl.ir` and `AudioControl.ar` expand the values. Each instance
  holds numbers, and adds its control at the special index of the next
  parameter. A `Control` or a `TrigControl` gives its size to a last name
  without a default.
- The bodies of `LagControl.kr` and `LagControl.ar` copy fact 11. A lag can
  be an output.
- A call that fails also gives back the controls, the names, the sizes, the
  last name and the table of `NamedControl` as they were.
- `variant` takes the last entry of a name, as the map of sclang does. Its
  check of the size also counts the room left in the parameters (fact 13).
  `argument` searches only the arguments of `new`.

### SynthDef.wrap and prependArgs

- `SynthDefBuilder::wrap` takes arguments and a closure. It checks the
  arguments as `new` does, adds their groups of rate, and calls the closure
  with one input per argument. Around the closure, the build has no last
  name, as `controlNames` is nil around the function. A closure that fails
  leaves the build as it was.
- `new` and `wrap` share the loop over the groups of rate.
- `prependArgs` need no API. The caller hands their values to its own code,
  and gives `new` and `wrap` only the arguments that become controls.

### NamedControl

- A body for `ar`, `kr`, `ir`, `tr` and `new` runs `NamedControl.new` and
  `init` as sclang writes them, over facts 1 to 7.
  - The lags become a tree of numbers, outputs and nils, and their rate
    decides whether they are fixed.
  - The values become a tree of numbers. The control takes them flat, and
    the result takes their shape.
- The builder holds the table of the names of the build: the values, the
  lags, the rate, the fixed lag and the control of each name.
- The body reuses the helpers of the control UGens. A `Lag` takes the rate
  of its input and the rule of `Lag` in the catalogue.
- A spec name gives its default from `catalog::spec`.

### The error

`BuildError::BadControl { class, message }` refuses a second use with other
values, another rate or other fixed lags. It also refuses an unknown spec
name, a 0 inside an array of lags, and values and lags of `LagControl` that
differ in size. Its message names the case. The existing errors refuse the
rest:

- `WrongKind`: an output among the values.
- `EmptyInput`: no values.
- `MissingArgument`: no name.
- `Limit(NoSuchParameter)`, at `finish`: a name past the parameters.

## Differences from sclang that remain

The crate refuses each of these:

- A spec as an array or a `ControlSpec`.
- A `NamedControl` without a name, which sclang names `nil`.
- Values and lags of `LagControl.kr` that differ in size. If nothing reads
  the result, sclang skips the call.
- `LagControl.kr` with no values.

## Tests

- `generator/fixtures.scd` writes four files.
  - `controlugens` holds `Control.names` in seven cases, `Control.kr`
    without names and with nested values, and `TrigControl.ir`. It also
    holds `LagControl.kr` in six cases, `LagControl.ar` in two, and a
    variant on the last name of `Control.names`.
  - `defwrap` holds `SynthDef.wrap` in six cases, a variant on two entries
    of one name, and `prependArgs` of a number and of an array.
  - `specs` holds one argument per spec of the table, each without a
    default, with `metadata: (specs: ())`.
  - `namedcontrol` holds the cases of facts 1 to 9, and
    `namedcontrol_markup`, the graph of the test of sc-app3.
- `build` of the script takes `prependArgs` and the metadata. `refused`
  also takes a throw of `buildUgenGraph` or of the write, a name past the
  parameters, and a variant that writes no values.
- Each refused case must break in sclang, and a test of the crate expects
  its error. The cases are other values, another rate, other fixed lags,
  fixed then output lags, and a 0 in each kind of array of lags. Then an
  unknown spec, a second spec, an output among the values, empty values,
  and `Control.kr([])`. Last come a read `LagControl.kr` with sizes that
  differ, the two variants of fact 13, and names past the parameters.
- `tests/common/controls.rs` builds each graph, in the order of sclang.
  `NAMES` gains the four files, and `tests/golden.rs` counts 81 files.
- A test of the catalogue counts the specs, and checks `freq` and `db`.
  `tests/readme.rs` keeps the list of methods that `ugen` refuses: seven
  methods leave it.
- `tests/scsynth.rs` loads the new graphs. A measure in NRT shows which
  entry of a name `/s_new` sets.

## Steps

Each step is one commit on `feat/controls`. Every commit leaves the eight
commands of `CLAUDE.md` green.

1. `docs: the plan of version 0.8`.
2. `test: the golden files of 0.8, written by sclang 3.14.1`, with the
   measure of two entries of one name.
3. `feat: the control UGens in a build`: the four classes of controls in the
   catalogue, their bodies, `control_names`, the sizes, the last name, and
   `variant` and `argument`. Also `BuildError::BadControl` for
   `LagControl`, and the parity of `controlugens`.
4. `feat: SynthDef.wrap`: `wrap`, the shared loop, and the parity of
   `defwrap`.
5. `feat: the specs of sclang`: the table, `ControlSpec`, `spec` and
   `specs`, and the parity of `specs`.
6. `feat: NamedControl`: the entry in the catalogue, the body, the table of
   the build, the refusals, and the parity of `namedcontrol`.
7. `docs: the release of version 0.8`.

## sc-app3

After the tag, sc-app3 takes `v0.8.0` on a branch
`build/scsynth-synthdef-0.8.0` from `develop`. The submodule moves, and
`Cargo.lock` changes one line. Core's definitions use only arguments, so
they keep their 2,301 bytes. A new test of sc-expression compiles markup
with `<sc-ugen type="NamedControl">` against `namedcontrol_markup`. The PR
goes to `develop`, and merges once its CI is green.

## After version 0.8

The work after version 0.8, in the order of its value for sc-app3:

1. `Env`, `VarLag`, and the pseudo-UGens that wait for it: `EnvGate`, and
   the `pvcalc` methods of `PV_ChainUGen`
2. sc3-plugins
