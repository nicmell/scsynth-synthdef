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
- Never touch the user's scsynth on UDP 57110. A test that needs scsynth runs
  its own in NRT mode, which opens no port.

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

The review of the plan against the sources found the facts 30 to 41. The
user then took these, also on 2026-10-18.

6. The builder keys the constants by the bits of the f64. It refuses NaN, as
   `checkValidInputs` does (fact 21). No parity fixture holds both 0.0 and
   -0.0.
7. The writer of version 1 stays. Its golden file comes from
   `SynthDefOld.newCopyArgs` (fact 30).
8. A class that the probe of the generator cannot map is not addable by name
   in 0.1. The catalogue records the reason, and the README lists these
   classes.
9. Version 0.1 checks the inputs with the generic rules of `checkInputs` and
   the rule of `AbstractOut` (facts 34 and 35). A class whose `checkInputs`
   belongs to another class is not addable by name.
10. The acceptance test runs scsynth in NRT mode, and only loads definitions
    (fact 40).
11. The catalogue keeps the categories of SCDoc, and gives a doc URL in place
    of the summary. No text of the help enters the crate (fact 41).
12. `add`, `ugen`, `unary_op` and `binary_op` take and return a `UGenInput`:
    a number, an output, or an array of them. The folds, `madd` and
    multichannel expansion of later versions then need no change of the API.

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
  `schedule`. Its checks are `checkInputs`, `checkValidInputs`,
  `checkNInputs` and `checkSameRateAsFirstInput`. The file also holds
  `PureUGen`, `MultiOutUGen` and `OutputProxy`.
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
  `UnitSpec_Read`, `calcParamSpecs`, `GraphDef_ReadVariant`,
  `DoBufferColoring` and `GraphDef_Recv`.
- `include/common/SC_Types.h` (`kSCNameLen`) and
  `server/scsynth/SC_SynthDef.h` (`kSCNodeDefNameLen`).
- `lang/LangSource/Opcodes.h` and `lang/LangSource/PyrParseNode.cpp`
  (`initSpecialSelectors`): the operator indices.

## Facts established on 2026-10-18

Each fact names its source. "Measured" means a run of sclang 3.14.1, or of
scsynth 3.14.1 where the fact says so.

### The file

1. All data is big endian and packed. A pstring is a u8 length and that many
   bytes. (format page)
2. Version 2 stores counts and indices as int32. Version 1 stores these as
   int16 instead (format page):
   - the counts of constants, parameters, names and UGens
   - the index of a name
   - the input and output counts of a UGen
   - both fields of an input spec
3. scsynth reads versions 2, 1 and 0. Versions 1 and 0 share
   `GraphDef_ReadVer1`, and version 0 has no variant section: the reader
   takes the count of variants only `if (inVersion >= 1)`
   (`SC_GraphDef.cpp:505`). scsynth ignores a file with a wrong magic or
   another version, and it reports no error. (`GraphDefLib_Read`)
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
10. `GraphDef_Recv` catches these exceptions and prints "exception in
    GraphDef_Recv: <reason>". No definition of the file loads: the exception
    leaves `GraphDefLib_Read` before it returns its list. `/d_recv` still
    replies `/done`, because `RecvSynthDefCmd::Stage4` sends it in any case
    (`SC_SequencedCommand.cpp:1353`).
11. scsynth does not check the bounds of an input spec: the UGen index, the
    output index and the constant index. An index out of range reads out of
    bounds. Any negative UGen index reads as a constant, with the output
    field as the index of the constant (`ReleaseInputBuffers`). scsynth reads
    the counts signed and stores them as uint32, so a negative count reads as
    about four billion. An unknown rate leaves the rate info of the unit
    unset. (`UnitSpec_Read`, `DoBufferColoring`, `GraphDef_Read`)
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
    each UGen in order. `addConstant` checks a `Set`, then stores the index
    in a `Dictionary`. Both look a Float up by its `hash`, then compare with
    `==`. sclang Floats are 64-bit, and `writeConstants` writes them as
    float32. Measured:
    - Two f64 values that round to the same f32 give two constants:
      `[0.0, 1.0, 1.0]`.
    - 0.0 and -0.0 have different hashes, but they are `==`, so the result
      depends on the layout of the two tables. `Out.kr(0, -0.0)` does not
      write: "SimpleNumber-writeInputSpec constant not found: -0.0". With 12
      other constants before them, 0.0 and -0.0 are two constants.
    - NaN never reaches the constants: `checkValidInputs` refuses it, "Out
      arg: 'channelsArray' has bad input: nan".
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
    0.43 s. `SCDoc.indexAllDocuments` writes `docmap.js` into
    `SCDoc.helpTargetDir` (`SCDoc.sc:511`), which is the help cache of the
    IDE by default.
29. Measured: sclang 3.14.1 knows 419 core UGen classes, and 234 of them
    define their own `*ar`, `*kr` or `*ir`. With the installed sc3-plugins
    it knows 917.

### Version 1 in sclang

30. `SynthDefOld.new` fails in 3.14.1. It calls `super.new`, which builds
    with a nil function: "Message 'def' not understood" (measured).
    `SynthDefOld.newCopyArgs(name: \o, metadata: (), children: Array(64))`
    followed by `build(func)` works, and its `asBytes` writes version 1
    (measured).

### The class library

31. The default configuration of sclang on this machine compiles the
    Extensions folder (sc3-plugins) and four quarks: Vowel, Dirt-Samples,
    scel and StrudelDirt (`sclang_conf.yaml`). A class extension can replace
    a method of a core class. `sclang -l <config>` with
    `excludeDefaultPaths: true` and the bundle's `SCClassLibrary` as its only
    include path compiles the core library alone. Measured: 419 UGen
    classes, no DFM1 and no Vowel.
32. Some defaults are expressions: `Delay1.kr` and `Delay2.kr` default `x1`
    to `(in)`, and their `prototypeFrame` holds nil there (measured). The
    `length` of Dseries, Dgeom, Dwhite and Dbrown defaults to `inf`
    (`Demand.sc`). Onsets maps the symbol `odftype` to a number
    (`MachineListening.sc`).

### What a rate method does besides `new1`

33. Measured: a probe of every core rate method (406 classes, 618 class and
    method pairs), once with distinct numbers as arguments and once with
    distinct control-rate outputs. 529 pairs create exactly one instance of
    their class, plus the `MulAdd` of `mul` and `add`. Besides the
    pseudo-UGens, these mechanisms account for the rest:
    - If `in.rate == \scalar` or `lagTime == 0`, Lag, Lag2, Lag3, LagUD,
      Lag2UD, Lag3UD and Ramp return `in.madd(mul, add)` (`Filter.sc:73`).
      With numbers, they create nothing.
    - `asAudioRateInput` puts a `DC.ar(x)` before a number and a `K2A.ar(x)`
      before a control-rate input: DelayN, DelayL, DelayC, CombN, CombL,
      CombC, AllpassN, AllpassL, AllpassC and their `Buf` forms
      (`Delays.sc`, `SimpleNumber.sc:595`).
    - `LocalBuf` creates one `MaxLocalBufs` per definition, and takes it as
      its third input. The input of `MaxLocalBufs` is the final count of
      LocalBufs (`BufIO.sc:150`).
    - `Poll` puts an `Impulse.kr(trig)` before a number, and makes its label
      from the class of its input.
    - `Out.ar`, `XOut.ar` and `LocalOut.ar` put a `DC` in place of each 0
      channel (`replaceZeroesWithSilence`, `UGen.sc:407`).
    - A channel count must be an Integer. Some arguments must be arrays,
      strings or symbols. `SendReply.kr` flops its arguments, so a String
      command name splits into characters.

### The checks of the inputs

34. `checkInputs` defaults to `checkValidInputs`, which refuses NaN. For a
    UGen at audio rate, `checkNInputs(n)` requires the first n inputs at
    audio rate. `checkSameRateAsFirstInput` requires the first input at the
    rate of the UGen. 39 UGen classes override `checkInputs`, in 15 files. At
    audio rate, `AbstractOut` refuses an input after its fixed arguments that
    is not audio rate. At control rate, it refuses a missing channel.
    Measured: "Out input at index 1 (0.5) is not audio rate".
35. scsynth does not check the rate of an input. `Out_Ctor` picks
    `Out_next_a` by the rate of the UGen, and `Out_next_a` copies a buffer's
    length of floats from each input (`IOUGens.cpp:944`). The buffer of a
    constant, and of a control-rate output, is one float
    (`&wire->mScalarValue`, `SC_Graph.cpp:364`), so it reads past it.

### More of scsynth

36. When the server runs, `DoBufferColoring` refuses a definition that needs
    more interconnect buffers than `-w` allows (default 64): "exceeded
    number of interconnect buffers." (`SC_GraphDef.cpp:909`). The count
    depends on the flag `kUnitDef_CantAliasInputsToOutputs` of each plugin.
37. An input from a later UGen reads an output that buffer coloring has not
    set yet: `ReleaseInputBuffers` calls `release(-1)`, which reads
    `refs[-1]` (`SC_GraphDef.cpp:835-859`).
38. Measured with NRT scsynth: a definition with a variant name of 32 bytes
    prints "exception in GraphDef_Recv: name too long (> 31 chars):
    v.abcdefghijklmnopqrstuvwxyzabcd", and one of 31 bytes loads. sclang
    writes the 32-byte name (fact 24). At 33 bytes, it writes a truncated
    file.
39. Measured: sclang writes several variants in the order of its
    dictionary. Given `b, a, zz, c, m`, it writes `c, a, b, zz, m`.
40. The scsynth of the bundle is built for JACK, without CoreAudio. If jackd
    does not run, `~/.bin/scsynth` exits. NRT mode (`-N`) opens no
    audio device and no port. Measured with NRT:
    - It prints the exceptions of `GraphDef_Recv`, and exits with 0 after it
      loads definitions.
    - `-U <bundle>/plugins` loads the core plugins only. `SC_PLUGIN_PATH` set
      to the same folder makes it abort: "libc++abi: terminating".
    - `/s_new` of a definition with an `AudioControl` crashes it (exit 139),
      for example `{ |a_in = 0| Out.ar(0, a_in) }`. The other control UGens
      run.
41. The help of SuperCollider is GPL-3, like its code: `COPYING` is the only
    license of the repository.

Also measured: for `{ |b = 1, a = #[]| }`, sclang writes one parameter and
the names b 0 and a 1. A name index can equal the number of parameters,
and scsynth then gives that name 0 channels (fact 12). `{ |a = #[]| }` alone
fails: "numChannels must be a nonzero positive integer, but received (0)".

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
- The checks of the inputs: `checkValidInputs`, `checkNInputs`,
  `checkSameRateAsFirstInput` and the rule of `AbstractOut`.
- An optional `serde` feature for the types of a definition.

### Out of scope: the todo list

- `optimizeGraph`: the MulAdd, Sum3 and Sum4 fusions, the rewrites of
  `a + b.neg` into `a - b` and of `a - b.neg` into `a + b`, and the removal of
  unused pure UGens. Also `addCopiesIfNeeded` for PV chains.
- Multichannel expansion (`multiNewList` with arrays).
- `mul` and `add` (`madd`, `MulAdd:new1`), and the folds of
  `BinaryOpUGen:new1` (fact 26).
- What rate methods do besides `new1` (fact 33): the folds of the Lag
  family, `asAudioRateInput`, `LocalBuf` with `MaxLocalBufs`, `Poll`, and
  the zeros of `Out`.
- `NamedControl`, `Control.names`, `SynthDef.wrap`, `prependArgs`, and the
  specs of the metadata.
- The pseudo-UGens, such as `Mix`, `Splay` and `SoundIn`, and `Env`.
- The `checkInputs` rules of the classes that define their own.
- The count of interconnect buffers (fact 36).
- sc3-plugins.
- The view of `SynthDesc`: inputs, outputs and metadata.

Version 0.1 matches the bytes of sclang for any graph that these do not
change. The graph must also have at most one variant, and not both 0.0 and
-0.0. The parity tests use only such graphs.

## Design

### Layout

```
Cargo.toml
src/lib.rs                the README as crate docs, and the re-exports
src/rate.rs               Rate
src/synth_def.rs          SynthDef, UGen, InputSpec, ControlName, Variant
src/file.rs               reading and writing a file, and its versions
src/limits.rs             the limits of scsynth and the format's invariants
src/error.rs              the errors
src/builder.rs            SynthDefBuilder: arguments, UGens, finish
src/ugen_input.rs         UGenInput and OutputProxy
src/topo.rs               the topological sort
src/check_inputs.rs       the rules of checkInputs
src/operator/mod.rs       the operators by name and index
src/operator/generated.rs the operator tables (generated)
src/catalog/mod.rs        UGenClass and lookup
src/catalog/generated.rs  the catalogue (generated)
generator/sclang_conf.yaml the core class library alone, for sclang -l
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
  and scscynth reads it. Its fields:
  - `name: String`
  - `constants: Vec<f32>`
  - `controls: Vec<f32>`, the initial values of the parameters
  - `control_names: Vec<ControlName>`, the names table in file order, each
    a `name` and an `index`
  - `children: Vec<UGen>`, in sorted order. The position of a child is its
    `synthIndex`.
  - `variants: Vec<Variant>`, each a full `name` and one value per parameter
- `UGen`: `name` (the class), `rate`, `inputs: Vec<InputSpec>`,
  `outputs: Vec<Rate>` (one rate per output) and `special_index: i16`.
- `InputSpec`: `Constant(index)`, or `UGen { index, output }`. It is the
  "input-spec" of the format page.
- `DefFile`: a `version` (`V0`, `V1` or `V2`) and its definitions.
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
  needs the sorted order (fact 37)
- an output index past the outputs of its UGen, and a constant index past
  the constants
- a name index past the number of parameters, and a variant with another
  number of values. A name index equal to the number of parameters is
  valid (the measurement after fact 41).
- a count that does not fit the signed integer of the version. The counts
  of definitions and of variants are int16 in every version.

Reading is strict about structure. It refuses:

- a truncated file, and trailing bytes
- a negative count, and a negative index other than the -1 of a constant
- a wrong magic, and an unknown version
- an unknown rate
- a name that is not UTF-8

scsynth ignores a wrong magic and an unknown version (fact 3), but the crate
refuses them, and the README says so.

### The builder

`SynthDefBuilder` is the `SynthDef` of sclang during a build.

- `SynthDefBuilder::new(name, args)` takes the arguments of the graph
  function. It builds their controls at once, as `buildUgenGraph` does
  before it calls the function (facts 16 to 18).
  - An `Arg` has a `name`, a default, an optional rate and an optional lag.
    The default is a number or an array of numbers, as f64. The rate is
    `Ir`, `Tr`, `Ar` or `Kr`. The lag is a number, or one per value of an
    array.
  - The rate follows fact 16. The optional rate plays the part of the
    entry in `rates`, and the prefix of the name still counts.
  - It gives one `UGenInput` per argument: one output, or an array of
    outputs for an array.
  - The builder refuses a lag on a scalar, trigger or audio argument. sclang
    ignores it with a warning. The README lists this difference.
  - The builder refuses two arguments with the same name, as a function of
    sclang does, and an empty array.
- A `UGenInput` is a number (f64, like a Float of sclang), an
  `OutputProxy` (a UGen and one of its outputs), or an array of them. An
  `OutputProxy` gives its rate, for a caller that picks `ar` or `kr`.
- `add` adds any UGen, at the level of `UGen:new1` after `init`. It takes a
  class name, a rate, the inputs, the rates of the outputs, a special
  index, and whether the class is a `WidthFirstUGen`. It refuses an array,
  which is multichannel expansion, and NaN.
- `ugen` adds a UGen by its class name and a method selector (`ar`, `kr`,
  `ir` or `new`), with the arguments by name. The catalogue gives the order
  of the arguments, their defaults, the wire slots, the outputs, the special
  index and the rule of `checkInputs`. Version 0.1 refuses:
  - a value for `mul` or `add` other than its default (1 and 0)
  - an array on a scalar argument, which is multichannel expansion
  - a class that the catalogue marks as not addable by name
- `unary_op` and `binary_op` add a `UnaryOpUGen` and a `BinaryOpUGen` with
  the special index of the operator and the rate of fact 26. They do not
  fold (fact 26). The folds are on the todo list.
- `variant(name, values)` adds a variant by control name, like the
  `variants` argument of `SynthDef.new`. Version 0.1 writes the variants in
  the order of the calls (fact 39).
- `finish` runs `collectConstants` (fact 21), the topological sort (fact 22)
  and `indexUGens`. It returns a `SynthDef` that meets the limits.

The constants follow fact 21, keyed by the bits of the f64 (decision 6).

### The catalogue

`catalog::lookup(name)` returns a `UGenClass`. `catalog::all()` returns all
of them, sorted by name.

- `UGenClass`:
  - `name`, and `superclasses` up to `UGen`
  - `is_pure` (`PureUGen` or `PureMultiOutUGen`) and `is_width_first`
    (`WidthFirstUGen`)
  - `check_inputs`: the rule of its `checkInputs`
  - `methods: &[RateMethod]`
  - `categories`, from SCDoc, and `doc_url()`
- `RateMethod`:
  - `selector` (`ar`, `kr`, `ir` or `new`), and the `rate` it builds at
  - `arg_names` and `defaults`, the `argNames` and the `prototypeFrame` of
    the method in sclang
  - `inputs: &[InputSlot]`
  - `outputs`, `madd`, and whether `ugen` can add it, or why not
- `InputSlot`:
  - `Arg(i)`: a scalar argument
  - `ArgArray(i)`: an array argument, flattened in place
  - `ArgString(i)`: a string, written as its size and its ASCII codes (fact
    27)
- `Outputs`: `Fixed` with the rate of each output, `Arg(i)` for a count
  from an argument such as `numChannels`, or `ArraySize(i)`.
- A class whose wire layout fits none of these stays in the catalogue with
  its arguments, but `ugen` refuses it. `Klank` is one (fact 27). The README
  lists these classes.

### The generator

`generator/catalog.scd` runs in sclang 3.14.1 and writes
`generator/catalog.json`. It runs as `sclang -l generator/sclang_conf.yaml
generator/catalog.scd`, so only the core class library compiles (fact 31).

1. It refuses any version other than 3.14.1 (`Main.version`), and any class
   or method from a file outside `Platform.classLibraryDir`.
2. It takes the subclasses of `UGen`.
3. For each class, it looks up the methods `ar`, `kr`, `ir` and `new` that
   the class responds to. It keeps a method that a UGen class owns, and it
   drops the `new` of `Object`. It records the argument names and the
   defaults.
4. A committed table gives the kind of some arguments, keyed by the class
   that owns the method, the selector and the argument: a count, an array or
   a symbol. Every other argument is a signal.
5. It probes each method twice, each time in a build of its own. First with
   numbers: a distinct non-integer for each signal, 2 for a count, an array
   of two distinct numbers, and the symbol `'/abcd'` (fact 33). Then with
   distinct outputs of one `Control.kr` for the signals.
6. After each call, it reads the new UGens from `UGen.buildSynthDef.children`,
   before `finishBuild` runs. It maps each input back to its argument. Both
   calls must create exactly one instance of the class, plus the `MulAdd` or
   `BinaryOpUGen` of `mul` and `add`. Both must give the same input map,
   rate and special index. Then the method is addable.
7. One more call per count or array argument, with that one at 3 and the
   others at 2, finds whether the output count follows it.
8. A probe that fails goes into the JSON with its reason. So does a probe
   that creates no instance of the class, or more than one.
9. A committed table maps the class that owns each `checkInputs` to a rule.
   An owner outside the table makes the generator fail.
10. It adds the categories from SCDoc (fact 28), with `SCDoc.helpTargetDir`
    set to a temporary folder first.
11. It writes the operator tables. The names come from
    `initSpecialSelectors`, and the order from `Opcodes.h`. The
    `specialIndex` of sclang must agree with each one.

The JSON tags its values (`{"float": 0.5}`, `{"float": "inf"}`, and so on),
because JSON has no inf. A test renders `generator/catalog.json` into
`src/catalog/generated.rs` and `src/operator/generated.rs`, and compares the
result with the committed files. With `SCSYNTH_SYNTHDEF_BLESS=1` set, it
writes them. So CI needs no sclang, and the JSON is the only output of the
generator. The generated files sit under `#[rustfmt::skip] mod generated;`.

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
  - an FFT chain, with `WidthFirstUGen`s, on a buffer number
  - demand UGens, at rate 3
  - repeated constants and `inf`
  - unary and binary operators
  - one variant
  - two definitions in one file
  - a file of version 1, from `SynthDefOld.newCopyArgs`
  - `EnvGen` with an `Env`, and `SendReply` with a string, both through
    `ugen`
  - an empty array beside another control, read only
- The acceptance by scsynth runs locally, outside CI:
  - A test marked `#[ignore]` runs scsynth 3.14.1 in NRT mode, with
    `-U <bundle>/plugins` and `-D 0`.
  - Its score sends each fixture compiled by the crate with `/d_recv`. It
    expects exit status 0, and no "exception in GraphDef_Recv" in the output
    of scsynth. It sends no `/s_new` (fact 40).
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
5. The operator tables, with the operator part of `generator/catalog.scd`,
   and `unary_op` and `binary_op`.
6. The classes in `generator/catalog.scd`, `catalog.json`, the rendered
   Rust, and the test that keeps them in step.
7. `ugen` by class name, with the rules of `checkInputs`, and parity tests
   through it.
8. The acceptance test with scsynth. Its measured limits go into the
   README.
9. The README, the CHANGELOG and the tag `v0.1.0`. Publishing stays out of
   scope.

The steps go in four batches, each on its own branch, stacked on the one
before: `feat/codec` (steps 2 and 3), `feat/builder` (4 and 5),
`feat/catalog` (6 and 7) and `feat/acceptance` (8 and 9). One PR from the
last branch takes them to `main`, with the skeleton.

Then sc-app3 takes the crate: the submodule `crates/scsynth-synthdef` with a
relative URL, a workspace member, and the CI step that fetches it. That work
is a branch of sc-app3, with its own plan.

## To measure while building

Nothing is open. The review of 2026-10-18 measured each item:

- the constants (fact 21)
- a variant name of 32 bytes (fact 38)
- an input from a later UGen (fact 37)
- the order of several variants (fact 39)

## After version 0.1

The out-of-scope list, in the order of its value for sc-app3:

1. `mul` and `add`, with the folds of `BinaryOpUGen:new1`
2. multichannel expansion
3. `optimizeGraph`
4. the extras of the rate methods (fact 33)
5. `NamedControl`
6. `Env` and the pseudo-UGens
7. the other `checkInputs` rules
8. sc3-plugins
