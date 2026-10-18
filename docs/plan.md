# The plan of scsynth-synthdef

This document is the plan of version 0.1. A new session starts from it, in
this folder, with no other context. The plan owns its decisions and facts
until the code and the tests own them. Then the facts move to the tests, the
README and the CHANGELOG, and this file keeps only what is still open.

## What we build

`scsynth-synthdef` is a Rust crate that writes and reads SuperCollider synth
definitions. It compiles a graph of UGens into the SynthDef file format
(`SCgf`), the bytes that scsynth loads with `/d_recv`. It reads those files
back. It carries a catalogue of the UGens of SuperCollider's core class
library.

- The crate is independent, and it can be published one day. The name was
  free on crates.io on 2026-10-18.
- Its first consumer is sc-app3 (github.com/nicmell/sc-app3). sc-app3 imports
  it as a git submodule, as it does with the sibling crate scsynth-osc
  (github.com/nicmell/scsynth-osc).
- The page of sc-app3 runs on wasm32, so the crate must build for
  `wasm32-unknown-unknown`. The crate does no I/O: bytes in, bytes out.
- The reference is SuperCollider 3.14.1: its file format, its sclang classes
  `SynthDef` and `UGen`, and its scsynth reader. Types and names follow
  sclang's where Rust allows.

## Ground rules

- This is a clean implementation from the SuperCollider sources. Do not
  consult earlier compilers: neither `sc-app2/packages/synthdef-compiler` nor
  `sc-crates/crates/scsynthdef-compiler`.
- A fact about SuperCollider comes from its source or from a measurement with
  sclang or scsynth 3.14.1. Each fact names its source: a file and a method,
  or the measurement.
- Where sclang and scsynth disagree, the crate refuses what scsynth refuses.
  The README lists each difference.
- Never touch the user's scsynth on UDP 57110. A test that needs scsynth
  boots its own on another port.

## Decisions

The user took these on 2026-10-18.

1. The name is `scsynth-synthdef`.
2. The registry is the UGen catalogue: what each UGen class takes and gives.
3. The catalogue covers SuperCollider core only. sc3-plugins go to the todo
   list. A generator builds the catalogue from sclang 3.14.1 and its class
   library.
4. Version 0.1 compiles like sclang, without graph rewriting and without
   multichannel expansion. It keeps sclang's control layout, its constant
   order and its topological sort.
5. Version 0.1 does synth definitions only: no envelopes (`Env`) and no
   pseudo-UGens.

## Sources of truth

- The format: https://doc.sccode.org/Reference/Synth-Definition-File-Format.html
- The UGen pages: https://doc.sccode.org/Browse.html#UGens
- The SuperCollider source at the tag `Version-3.14.1`, checked out in
  `~/Projects/supercollider`. If it is missing, clone
  github.com/supercollider/supercollider and check out the tag.
- The installed SuperCollider 3.14.1: `sclang` and `scsynth` on the PATH
  (`~/.bin`), and `/Applications/SuperCollider.app` with the class library
  and the help sources. `sclang -v` prints `sclang 3.14.1`. The user's
  Extensions folder holds sc3-plugins, which the catalogue leaves out.

The files of the source, and what each one owns:

- `SCClassLibrary/Common/Audio/SynthDef.sc`: the build. `buildUgenGraph`,
  `addControlsFromArgsOfFunc`, `buildControls`, `finishBuild`, `addUGen`,
  `addConstant`, `collectConstants`, `initTopoSort`, `topologicalSort`,
  `indexUGens`, `writeDef` and `writeConstants`.
- `SCClassLibrary/Common/Audio/UGen.sc`: the UGen. Its creation is `new1`,
  `multiNewList` and `addToSynth`. Its writing is `collectConstants`,
  `rateNumber`, `writeDef`, `writeInputSpec` and `writeOutputSpecs`. Its
  sort is `initTopoSort`, `makeAvailable`, `removeAntecedent` and
  `schedule`. The file also holds `PureUGen`, `MultiOutUGen` and
  `OutputProxy`.
- `SCClassLibrary/Common/Audio/InOut.sc`: `ControlName`, `Control`,
  `AudioControl`, `TrigControl` and `LagControl`, and the `In` and `Out`
  families.
- `SCClassLibrary/Common/Audio/BasicOpsUGen.sc`: `UnaryOpUGen`,
  `BinaryOpUGen`, `MulAdd`, `Sum3` and `Sum4`.
- `SCClassLibrary/Common/Audio/FFT.sc`: `WidthFirstUGen`.
- `SCClassLibrary/Common/Audio/SynthDefOld.sc`: the writer of version 1.
- `SCClassLibrary/Common/Collections/Collection.sc`: `writeDef`, the header
  of a file with several definitions.
- `SCClassLibrary/Common/Math/SimpleNumber.sc`: `writeInputSpec`, the input
  spec of a number.
- `server/scsynth/SC_GraphDef.cpp`: the reader. `GraphDefLib_Read`,
  `GraphDef_Read`, `GraphDef_ReadVer1`, `ReadName`, `ReadNodeDefName`,
  `UnitSpec_Read`, `calcParamSpecs`, `GraphDef_ReadVariant` and
  `GraphDef_Recv`.
- `include/common/SC_Types.h` (`kSCNameLen`) and
  `server/scsynth/SC_SynthDef.h` (`kSCNodeDefNameLen`).
- `lang/LangSource/Opcodes.h` and `lang/LangSource/PyrParseNode.cpp`
  (`initSpecialSelectors`): the operator indices.

## Facts established on 2026-10-18

Each fact names its source. "Measured" means a run of sclang 3.14.1.

### The file

1. All data is big endian and packed. A pstring is a u8 length and that many
   bytes. (format page)
2. Version 2 stores counts and indices as int32. Version 1 stores these as
   int16 instead (format page):
   - the counts of constants, parameters, names and UGens
   - the index of a name
   - the input and output counts of a UGen
   - both fields of an input spec
3. scsynth reads versions 2, 1 and 0. It reads version 0 with the layout of
   version 1. It ignores a file with a wrong magic or another version, and it
   reports no error. (`GraphDefLib_Read`)
4. A file holds an int16 count of definitions. sclang writes several
   definitions in one file with `Collection:writeDef`, for example
   `[a, b].writeDefFile("name")`. (`Collection.sc`, `Object:writeDefFile`)
5. The rates are 0 scalar, 1 control, 2 audio and 3 demand. The format page
   lists only 0 to 2. sclang writes 3 for demand (`UGen:rateNumber`), and
   scsynth reads it (`calc_DemandRate` in `GraphDef_Read`).
6. Each output has its own rate byte. A `MultiOutUGen` writes the rate of
   each `OutputProxy`. A plain UGen writes its own rate. An `AbstractOut`
   writes no output. (`UGen.sc`, `InOut.sc`)

### The limits of scsynth

7. A definition name must be shorter than 256 bytes (`ReadNodeDefName`,
   `kSCNodeDefNameByteLen`). A pstring holds at most 255 bytes, so any name
   fits.
8. A UGen class name, a parameter name and a variant name must be shorter
   than 32 bytes (`ReadName`, `kSCNameByteLen`). Otherwise scsynth throws
   "name too long (> 31 chars)".
9. A UGen that scsynth does not know throws "UGen '<name>' not installed."
   (`UnitSpec_Read`)
10. `GraphDef_Recv` catches these exceptions and prints them. The definition
    does not load.
11. scsynth does not check the bounds of an input spec: the UGen index, the
    output index and the constant index. An index out of range reads out of
    bounds. An unknown rate leaves the rate info of the unit unset.
    (`UnitSpec_Read`, `DoBufferColoring`, `GraphDef_Read`)
12. scsynth derives the channel count of a parameter from the names table.
    It sorts the indices. Each name spans up to the next index, and the last
    one spans up to the number of parameters. (`calcParamSpecs`)
13. A variant holds one value per parameter. scsynth reads its name with
    `ReadName`, so fact 8 applies. (`GraphDef_ReadVariant`)

### The build in sclang

14. `finishBuild` runs `addCopiesIfNeeded`, `optimizeGraph`,
    `collectConstants`, `checkInputs`, `topologicalSort` and `indexUGens`,
    in this order.
15. `addUGen` gives each new UGen its `synthIndex`, the number of UGens so
    far. It also gives it `widthFirstAntecedents`, a copy of the
    `WidthFirstUGen`s created so far.
16. The arguments of the graph function become controls
    (`addControlsFromArgsOfFunc`):
    - The rate is the first match in the order scalar, trigger, audio. An
      argument matches a rate by its entry in `rates`: `\ir`, `\tr` or
      `\ar`. It also matches by the prefix of its name: `i_`, `t_` or `a_`.
      Without a match, the rate is control. So the prefix `i_` wins over an
      entry `\kr`.
    - In `rates`, `\kr` and a number both mean control rate. A number is the
      lag.
    - A nil default becomes 0.0.
    - A lag on a scalar, trigger or audio argument posts a warning, and
      sclang ignores it.
17. `buildControls` makes one control UGen per rate group, in this order:
    `Control` at scalar rate, `TrigControl`, `AudioControl`, and `Control`
    at control rate. When any lag is not 0, the control group becomes
    `LagControl`. `LagControl:kr` clumps its values by 16, one `LagControl` per
    clump. The parameter indices follow the order of the groups, not the
    order of the arguments.
18. A control UGen has one output per value, at its own rate, and no inputs.
    The inputs of a `LagControl` are its lags. The special index of a control
    UGen is the index of its first value. (`Control:init`,
    `LagControl:init`)
19. `writeDef` writes the names table in the order of `allControlNames`,
    which is the order of the arguments. It skips `noncontrol` names and nil
    names.
20. Measured: `{ |freq = 440, a_in = 0, i_n = 1, t_g = 0, amps = #[1, 2]| }`
    gives these control UGens and values:
    - `Control` at scalar rate, special index 0
    - `TrigControl`, special index 1
    - `AudioControl`, special index 2
    - `Control` at control rate with 3 outputs, special index 3
    - the parameter values `[1, 0, 0, 440, 1, 2]`
    - the names table freq 3, a_in 2, i_n 0, t_g 1, amps 4
21. `collectConstants` walks the UGens in creation order, and the inputs of
    each UGen in order. `addConstant` keeps the first of equal values, in a
    `Set` of sclang Floats, which are 64-bit. `writeConstants` writes them as
    float32.
22. The topological sort (`initTopoSort`, `topologicalSort`,
    `UGen:schedule`) runs in four steps:
    1. The antecedents of a UGen are the sources of its UGen inputs and its
       `widthFirstAntecedents`. Its descendants are the reverse relation.
       Both are sets, so a source read twice counts once. The descendants
       are then sorted by `synthIndex`.
    2. sclang walks the UGens in reverse creation order. It appends each one
       without antecedents to the list `available`.
    3. While `available` is not empty, sclang pops its last UGen. It walks
       that UGen's descendants in reverse order and removes the UGen from
       their antecedents. A descendant left without antecedents goes to the
       end of `available`. The popped UGen goes to the end of the output.
    4. `indexUGens` sets each `synthIndex` to the position of the UGen in the
       output.
23. The input spec of a UGen input is the `synthIndex` of its source and the
    output index, which is 0 for a plain UGen (`UGen:writeInputSpec`). The
    input spec of a number is -1 and the index of the constant. A number
    missing from the constants throws "SimpleNumber-writeInputSpec constant
    not found". (`SimpleNumber:writeInputSpec`)
24. sclang writes a variant as `<definition name>.<variant name>`, with one
    value per parameter. It refuses a name longer than 32 characters, but
    scsynth refuses 32 already (fact 8). (`SynthDef:writeDef`)
25. The special index of `UnaryOpUGen` and `BinaryOpUGen` is the
    `specialIndex` of the operator: its position in the enums of
    `Opcodes.h`. The names come from `initSpecialSelectors`. A negative
    index is an error: "Operator '%' applied to a UGen is not supported in
    scsynth". (`BasicOpUGen:operator_`) Measured: `+` 0, `*` 2, `neg` 0,
    `midicps` 17.
26. The rate of a `UnaryOpUGen` is the rate of its input
    (`UnaryOpUGen:init`). The rate of a `BinaryOpUGen` is the highest rate
    of its inputs, in the order demand, audio, control, scalar
    (`determineRate`).
    `BinaryOpUGen:new1` folds some cases before any UGen exists: `* 0`,
    `* 1`, `* -1`, `+ 0`, `- 0`, `0 -`, `/ 1` and `/ -1`.
27. The wire inputs of a UGen are not always its arguments in order.
    Measured:
    - `SinOsc.ar(freq, phase, mul, add)` gives the inputs `[freq, phase]`.
      `mul` and `add` make a `MulAdd`.
    - `In.ar(bus, numChannels)` gives the inputs `[bus]` and `numChannels`
      outputs.
    - `EnvGen.kr(envelope, gate, levelScale, levelBias, timeScale,
      doneAction)` gives `[gate, levelScale, levelBias, timeScale,
      doneAction]`, then `envelope.asArray`.
    - `Klank.ar(specificationsArrayRef, input, freqscale, freqoffset,
      decayscale)` gives `[input, freqscale, freqoffset, decayscale]`, then a
      frequency, an amplitude and a ring time for each resonator.
    - `SendReply.kr(trig, cmdName, values, replyID)` gives `[trig, replyID,
      cmdName.size]`, then the ASCII codes of `cmdName`, then `values`.
28. SCDoc in sclang gives the summary and the categories of each class. For
    SinOsc: "Interpolating sine wavetable oscillator." and
    `UGens>Generators>Deterministic`. Measured: indexing 1965 documents took
    0.43 s.
29. Measured: sclang 3.14.1 knows 419 core UGen classes, and 234 of them
    define their own `*ar`, `*kr` or `*ir`. With the installed sc3-plugins
    it knows 917.

## Version 0.1

### In scope

- The codec: read versions 0, 1 and 2, with several definitions per file and
  their variants. Write version 2, and version 1 like `SynthDefOld`.
- The limits of scsynth and the invariants of the format, applied to every
  definition that the crate reads or writes.
- The builder: the build of sclang without `optimizeGraph`. Controls from
  arguments, UGens, constants, the topological sort and variants.
- The operators: `UnaryOpUGen` and `BinaryOpUGen`, with their special
  indices and rates.
- The catalogue of the core UGens, generated, and a builder step that adds a
  UGen by its class name.
- An optional `serde` feature for the types of a definition.

### Out of scope: the todo list

- `optimizeGraph`: the MulAdd, Sum3 and Sum4 fusions, and the removal of
  unused pure UGens. Also `addCopiesIfNeeded` for PV chains.
- Multichannel expansion (`multiNewList` with arrays).
- `mul` and `add` (`madd`, `MulAdd:new1`), and the folds of
  `BinaryOpUGen:new1` (fact 26).
- `NamedControl`, `Control.names`, `SynthDef.wrap`, `prependArgs`, and the
  specs of the metadata.
- The pseudo-UGens, such as `Mix`, `Splay` and `SoundIn`, and `Env`.
- The `checkInputs` rules of each class.
- sc3-plugins.
- The view of `SynthDesc`: inputs, outputs and metadata.

Version 0.1 matches the bytes of sclang for any graph that these do not
change. The parity tests use only such graphs.

## Design

### Layout

```
Cargo.toml
src/lib.rs                the README as crate docs, and the re-exports
src/rate.rs               Rate
src/synth_def.rs          SynthDef, UGen, Input, ControlName, Variant
src/file.rs               reading and writing a file, and its versions
src/limits.rs             the limits of scsynth and the format's invariants
src/builder.rs            SynthDefBuilder: arguments, UGens, finish
src/topo.rs               the topological sort
src/operator.rs           the operator tables (generated)
src/catalog/mod.rs        UGenClass and lookup
src/catalog/generated.rs  the catalogue (generated)
src/error.rs              Error
generator/catalog.scd     the sclang generator of the catalogue and the operators
generator/catalog.json    its output, committed
generator/fixtures.scd    the sclang generator of the golden files
tests/fixtures/           the golden .scsyndef files, committed
tests/*.rs                the integration tests
```

The module names are a starting point. The implementer can merge or split
modules.

### Types

The types follow the names of sclang. The signatures are sketches.

- `Rate`: `Scalar`, `Control`, `Audio` and `Demand`, with `rate_number` (0
  to 3) and its inverse.
- `SynthDef`: a finished definition. sclang writes it after `finishBuild`,
  and scsynth reads it. Its fields:
  - `name: String`
  - `constants: Vec<f32>`
  - `controls: Vec<f32>`, the initial values of the parameters
  - `control_names: Vec<ControlName>`, the names table in file order, each
    a `name` and an `index`
  - `children: Vec<UGen>`, in sorted order. The position of a child is its
    `synthIndex`.
  - `variants: Vec<Variant>`, each a full `name` and one value per parameter
- `UGen`: `name` (the class), `rate`, `inputs: Vec<Input>`,
  `outputs: Vec<Rate>` (one rate per output) and `special_index: i16`.
- `Input`: `Constant(index)`, or `UGen { synth_index, output_index }`.
- The functions:
  - `SynthDef::as_bytes`: a file of one definition, version 2, like the
    `asBytes` of sclang
  - `write_def_file(&[SynthDef])`: version 2, like `Collection:writeDef`
  - `write_def_file_old(&[SynthDef])`: version 1, like `SynthDefOld`
  - `read_def_file(&[u8])`: a `DefFile`, with its `version` and its
    definitions

### Limits and invariants

The crate refuses what scsynth refuses or cannot read safely. Writing
applies these rules first, and reading applies them after the parse:

- a definition name longer than 255 bytes
- a UGen class name, a parameter name or a variant name of 32 bytes or more
  (fact 8)
- an input from a UGen at or after the UGen that reads it, because scsynth
  needs the sorted order
- an output index past the outputs of its UGen, and a constant index past
  the constants
- a name index past the parameters, and a variant with another number of
  values
- a count that does not fit the integer of the version

Reading is strict about structure. It refuses a truncated file, trailing
bytes, a negative count, a wrong magic, an unknown version and an unknown
rate. scsynth ignores a wrong magic and an unknown version (fact 3), but the
crate refuses them, and the README says so.

### The builder

`SynthDefBuilder` is the `SynthDef` of sclang during a build.

- `SynthDefBuilder::new(name, args)` takes the arguments of the graph
  function. It builds their controls at once, as `buildUgenGraph` does
  before it calls the function (facts 16 to 18).
  - An `Arg` has a `name`, a default (a number or an array of numbers, as
    f64), an optional rate (`Ir`, `Tr`, `Ar` or `Kr`) and an optional lag.
  - The rate follows fact 16. The optional rate plays the part of the
    entry in `rates`, and the prefix of the name still counts.
  - It returns one handle per argument: one output, or one output per value
    for an array.
  - The builder refuses a lag on a scalar, trigger or audio argument. sclang
    ignores it with a warning. The README lists this difference.
  - The builder refuses two arguments with the same name, as a function of
    sclang does.
- An input is a number (f64, like a Float of sclang) or an `OutputProxy`: a
  UGen and one of its outputs.
- `add` adds any UGen, at the level of `UGen:new1` after `init`. It takes a
  class name, a rate, the inputs, the outputs as a count or as rates, a
  special index, and whether the class is a `WidthFirstUGen`.
- `ugen` adds a UGen by its class name and a method selector (`ar`, `kr`,
  `ir` or `new`), with the arguments by name. The catalogue gives the order
  of the arguments, their defaults, the wire slots, the outputs and the
  special index. Version 0.1 refuses:
  - a value for `mul` or `add` other than its default (1 and 0)
  - an array on a scalar argument, which is multichannel expansion
  - a class that the catalogue marks as not addable by name
- `unary_op` and `binary_op` add a `UnaryOpUGen` and a `BinaryOpUGen` with
  the special index of the operator and the rate of fact 26. They do not
  fold (fact 26). The folds are on the todo list.
- `variant(name, values)` adds a variant by control name, like the
  `variants` argument of `SynthDef.new`.
- `finish` runs `collectConstants` (fact 21), the topological sort (fact 22)
  and `indexUGens`. It returns a `SynthDef` that meets the limits.

The constants follow fact 21. Their keys are f64 values, compared like the
`==` of sclang. Measure three cases before the code fixes them: 0.0 against
-0.0, NaN, and two f64 values that round to the same f32.

### The catalogue

`catalog::lookup(name)` returns a `UGenClass`. `catalog::all()` returns all
of them, sorted by name.

- `UGenClass`:
  - `name`, and `superclasses` up to `UGen`
  - `is_pure` (`PureUGen` or `PureMultiOutUGen`) and `is_width_first`
    (`WidthFirstUGen`)
  - `methods: &[RateMethod]`
  - `categories` and `summary`, from SCDoc
- `RateMethod`:
  - `selector` (`ar`, `kr`, `ir` or `new`), and the `rate` it builds at
  - `arg_names` and `defaults`, the `argNames` and the `prototypeFrame` of
    the method in sclang
  - `inputs: &[InputSlot]`
  - `outputs` and `madd`
- `InputSlot`:
  - `Arg(i)`: a scalar argument
  - `ArgArray(i)`: an array argument, flattened in place
  - `ArgString(i)`: a string, written as its size and its ASCII codes (fact
    27)
- `Outputs`: `Fixed(n)`, `Arg(i)` for a count from an argument such as
  `numChannels`, or `ArraySize(i)`. It also holds the rate of each output
  when the rates differ.
- A class whose wire layout fits none of these stays in the catalogue with
  its arguments, but `ugen` refuses it. `Klank` is one (fact 27). The README
  lists these classes.

### The generator

`generator/catalog.scd` runs in sclang 3.14.1 and writes
`generator/catalog.json`.

1. It refuses any version other than 3.14.1 (`Main.version`).
2. It takes the subclasses of `UGen` that the class library of the running
   sclang defines. It leaves out the Extensions folder.
3. For each class, it looks up the methods `ar`, `kr`, `ir` and `new` that
   the class responds to. It keeps a `new` that a UGen class defines, and it
   drops the `new` of `Object`. It records the argument names and the defaults.
4. It probes each method inside a build, with sentinels: a distinct number
   for each argument, and an array of distinct numbers for an array
   argument.
5. After the call, it reads the new instance of the class from
   `UGen.buildSynthDef.children`, before `finishBuild` runs. It maps each
   input back to its argument. It also reads the outputs, the rate, the
   special index and the rate of each output.
6. It calls each argument like `numChannels` with two values, to see whether
   the output count follows it. `mul` and `add` show up as a `MulAdd` or a
   `BinaryOpUGen` on the instance (fact 27).
7. A probe that fails goes into the JSON with its reason. So does a probe
   that creates no instance of the class: a pseudo-UGen. Neither is addable
   by name.
8. It adds the summary and the categories from SCDoc (fact 28).
9. It writes the operator tables. The names come from
   `initSpecialSelectors`, and the order from `Opcodes.h`. The `specialIndex`
   of sclang must agree with each one.

A test renders `generator/catalog.json` into `src/catalog/generated.rs` and
`src/operator.rs`, and compares the result with the committed files. With
`SCSYNTH_SYNTHDEF_BLESS=1` set, it writes them. So CI needs no sclang, and
the JSON is the only output of the generator.

## Tests

- Unit tests for each module: each structural error of reading, each limit,
  and each step of the build.
- The golden files. `generator/fixtures.scd` builds a list of definitions in
  sclang 3.14.1 and writes each to `tests/fixtures/<name>.scsyndef`.
  - A test builds the same graph with the builder and compares the bytes.
  - Every golden file must read and write back to the same bytes.
  - If `optimizeGraph` changes a fixture, the script fails. It compares the
    UGens before and after `finishBuild`.
- The fixtures:
  - one UGen into `Out`, the `sine` definition
  - every argument rate, an array argument and lags
  - more than 16 lagged values, so two `LagControl`s
  - multi-output UGens: `Pan2`, and `In` with `numChannels`
  - a diamond, a fan-out, and a creation order that the sort changes
  - an FFT chain, with `WidthFirstUGen`s
  - demand UGens, at rate 3
  - repeated constants, -0.0 and `inf`
  - unary and binary operators
  - one variant
  - two definitions in one file
  - a file of version 1, from `SynthDefOld`
  - `EnvGen` with an `Env`, and `SendReply` with a string, both through
    `ugen`
- The acceptance by scsynth runs locally, outside CI:
  - A test marked `#[ignore]` boots `scsynth` 3.14.1 on a free UDP port,
    with its own synthdef folder.
  - It sends each fixture compiled by the crate with `/d_recv`. It expects
    `/done`, and no "exception in GraphDef_Recv" in the output of scsynth.
  - It encodes its few OSC messages by hand, so it needs no dependency.
  - The same test measures the limits of facts 8 and 24.

## Repository setup

- Run `cargo init --lib`. Use edition 2024 and `rust-version = "1.85"`.
- Copy the package fields of scsynth-osc and adapt them: description,
  keywords, categories, `include`, docs.rs with all features, and the lints.
  The lints are `missing_docs` warn, `unsafe_code` forbid, clippy `all` and
  `pedantic` warn, and `module_name_repetitions` allow.
- The crate has no runtime dependency. `serde` is optional, and
  `serde_json` is a dev-dependency.
- Copy these files from scsynth-osc and adapt them:
  - `LICENSE-MIT` and `LICENSE-APACHE`, for MIT OR Apache-2.0
  - `deny.toml`
  - `clippy.toml`, for `doc-valid-idents`
  - `.github/workflows/ci.yml`, with the jobs `checks`, `msrv` and `deny`
  - the format of `CHANGELOG.md`
- The README is the crate documentation
  (`#![doc = include_str!("../README.md")]`). Its sections follow
  scsynth-osc's: what the crate is, usage, features, traps and limits,
  differences from SuperCollider, and the license.
- `CLAUDE.md` is the operating manual. `AGENTS.md` is a committed relative
  symlink to it.
- The GitHub repository is `nicmell/scsynth-synthdef`, public like
  scsynth-osc.
- Every dependency must be at least 7 days old. Pin a newer one down with
  `cargo update -p <crate> --precise <version>`.
- Work goes on a branch, with one commit per step. Every commit leaves fmt,
  clippy, the tests and the docs green. The branch reaches `main` through a
  GitHub PR.

## Steps

1. Done on 2026-10-18. The skeleton: `Cargo.toml`, the licenses,
   `deny.toml`, `clippy.toml`, CI, a short README and the CHANGELOG.
2. `Rate`, the types of `SynthDef`, reading and writing versions 0, 1 and 2,
   the limits, and their unit tests.
3. `generator/fixtures.scd`, the golden files, and the round trip of each.
4. The builder: arguments and controls, `add`, the constants, the
   topological sort, the variants and `finish`. Parity tests against the
   golden files.
5. The operator tables, `unary_op` and `binary_op`.
6. `generator/catalog.scd`, `catalog.json`, the rendered Rust, and the test
   that keeps them in step.
7. `ugen` by class name, and parity tests through it.
8. The acceptance test with scsynth. Its measured limits go into the
   README.
9. The README, the CHANGELOG and the tag `v0.1.0`. Publishing stays out of
   scope.

Then sc-app3 takes the crate: the submodule `crates/scsynth-synthdef` with a
relative URL, a workspace member, and the CI step that fetches it. That work
is a branch of sc-app3, with its own plan.

## To measure while building

- 0.0 against -0.0 as constants, NaN, and two f64 values that round to the
  same f32 (fact 21)
- whether scsynth accepts a variant name of exactly 32 bytes (fact 24)
- what scsynth does with an input that reads a later UGen, which sclang
  never writes
- the order in which sclang writes several variants. `variants` is a
  dictionary, so its order is not the order of the source. Version 0.1
  writes the variants in the order of the calls.

## After version 0.1

The out-of-scope list, in the order of its value for sc-app3: `mul` and
`add` with the folds of `BinaryOpUGen:new1`, multichannel expansion,
`optimizeGraph`, `NamedControl`, `Env`, the pseudo-UGens, `checkInputs`, and
sc3-plugins.
