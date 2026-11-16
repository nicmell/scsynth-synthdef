# The plan of scsynth-synthdef

This document is the plan of version 0.10. A session starts from it and
from the README, in this folder. The plan owns its decisions and facts
until the code and the tests own them. Then the facts move to the tests,
the README and the CHANGELOG, and this file keeps only what is still open.

## What we build

Version 0.10 counts the wire buffers of a definition, as scsynth 3.14.1
counts them. sc-app3 asks for this count. scsynth answers `/done /d_recv`
to a definition that needs more wire buffers than its `-w`, but it holds
no definition, and only its console tells.

- `SynthDef::wire_buffers` gives the count of scsynth for each definition
  of the UGens of the catalogue.
- `DemandEnvGen.ar` writes the bytes of sclang. Today it lacks the `K2A`
  that sclang adds for an audio-rate `gate` or `reset`.

Then sc-app3 takes the release, and can count the buffers of the
definitions that it compiles.

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

1. `SynthDef::wire_buffers` gives the count that scsynth 3.14.1 computes.
   A UGen outside the catalogue gives an error, because the crate does not
   know its flag.
2. The generator reads the flags from the plugin sources at
   `Version-3.14.1` into the catalogue.
3. A fixture holds the count of every golden definition, as scsynth
   measures it in NRT mode. CI tests the crate against the fixture.
4. Golden cases cover each rule of the count.
5. sc-app3 takes 0.10 with the move of its submodule and a test of the
   count. A pump that refuses a definition over `-w` gets a plan of its own
   in sc-app3, because scsynth does not report its `-w`.
6. After 0.10 come `PackFFT.new` and the `pvcalc` methods, then the methods
   of `UGen` that build graphs, then sc3-plugins.
7. When the CI of the PR is green, the work stops. On the go of the owner,
   the PR merges, and `v0.10.0` becomes an annotated tag on its merge
   commit. Then sc-app3 moves, and its PR merges into `develop` once its CI
   is green.

## Sources of truth

The SuperCollider source at the tag `Version-3.14.1`, in
`~/Projects/supercollider`, and sclang and scsynth 3.14.1. In
`server/scsynth`: `SC_GraphDef.cpp` (`DoBufferColoring`), `SC_World.cpp`
and `SC_SequencedCommand.cpp`. In `include/plugin_interface`:
`SC_InterfaceTable.h`. In `server/plugins`: the plugins and their flags.
In `SCClassLibrary/Common/Audio`: `Demand.sc`.

## Facts established on 2026-11-16

The measured facts come from two programs. The scsynth of the bundle ran
in NRT mode, with the score piped on stdin. sclang 3.14.1 ran probes of
one line.

### The count of scsynth

1. scsynth walks the UGens in file order. Each audio-rate output takes a
   buffer, with one reference for each input that reads it. An output that
   one UGen reads twice has two. A UGen frees its references first, then
   takes buffers for its outputs, so an output can take a buffer that an
   input just freed. A freed buffer goes on a stack, and the next output
   takes it. The count, `mNumWireBufs`, is the most buffers in use at one
   time. Control-rate and scalar outputs take no buffer.
   - Measured: `Out.ar(0, LPF.ar(SinOsc.ar, 1000))` needs 1, and so does
     `s * s` of one `SinOsc`. `pair` and `Pan2.ar(SinOsc.ar)` need 2.
     `Out.kr(0, SinOsc.kr)` needs 0, and loads at `-w 0`.
   - 16 `SinOsc` into an `Out` of 16 channels need 16. sc-app3 measured
     that `-w 8` refuses this graph.
2. A unit that its plugin defines with `DefineSimpleCantAliasUnit` or
   `DefineDtorCantAliasUnit` has the flag
   `kUnitDef_CantAliasInputsToOutputs`. It takes buffers for its outputs
   before it frees its references. No plugin sets the flag in another way.
   18 core units have it, in six plugins:
   - `DelayUGens.cpp`: `PlayBuf` and `TGrains`.
   - `DemandUGens.cpp`: `Demand`, `DemandEnvGen`, `Duty` and `TDuty`.
   - `GrainUGens.cpp`: `GrainBuf`, `GrainFM`, `GrainIn`, `GrainSin` and
     `Warp1`.
   - `ML.cpp`: `BeatTrack` and `BeatTrack2`.
   - `PanUGens.cpp`: `Balance2`, `DecodeB2`, `PanAz` and `PanB2`.
   - `PartitionedConvolution.cpp`: `PartConv`.

   Measured, each unit with an audio input:
   - `Demand`, `Duty`, `TDuty`, `PlayBuf`, `Warp1` and `PartConv` need 2,
     and 1 without the flag.
   - `PanAz`, `TGrains`, `GrainSin`, `GrainFM` and `GrainBuf` need 3, and 2
     without the flag. `PanAz.ar(2, SinOsc.ar)` is one of them.
   - `DemandEnvGen` needs 3, and 2 without the flag. The count includes the
     `K2A` of fact 6.
   - `GrainIn` and `Balance2` need 4, and 2 without the flag. `PanB2` needs
     4, and 3 without it. `DecodeB2` needs 5, and 3 without it.
   - `PlayBuf.ar(1, 0)`, with no audio input, needs 1.
   - `BeatTrack` and `BeatTrack2` have control-rate outputs only, so the
     flag cannot change their count.
3. A demand-rate UGen never frees the buffers of its audio inputs.
   Measured: `Duty.ar(0.1, 0, Dseq([SinOsc.ar(1), 0], inf))` needs 2, and
   1 with `SinOsc.kr(1)`.
4. An output that nothing reads keeps its buffer to the end. Measured: this
   graph needs 3.

   ```supercollider
   var p = Pan2.ar(SinOsc.ar);
   Out.ar(0, p[0]);
   Out.ar(1, SinOsc.ar(2) * SinOsc.ar(3))
   ```

### The check of scsynth

5. `World_NonRealTimeSynthesis` calls `World_Start` before it runs the
   score. So NRT mode refuses a definition over `-w`, as a running server
   does: "exception in GraphDef_Recv: exceeded number of interconnect
   buffers."
   - Measured: scsynth then drops every definition of the file. A file of
     the graphs of fact 1 that need 1 and 16 loads neither at `-w 8`.
   - `/status.reply` holds no `-w`, so a client cannot ask for it.

### `DemandEnvGen.ar`

6. If `gate` or `reset` is at audio rate, `K2A.ar` takes the other one when
   it is not. The rate of an array is the least by name, as `lang::rate`
   gives it. Measured:
   - An audio gate adds `K2A(reset)`, and an audio reset adds `K2A(gate)`.
     Both at audio rate, or neither, add none.
   - `gate: [Impulse.ar(1), 1]` counts as audio. It adds one `K2A(reset)`
     before two `DemandEnvGen`.
   - An audio gate with `reset: [Impulse.kr(1), Impulse.kr(2)]` adds two
     `K2A`. `gate: \g.ar(1)` adds `K2A(reset)`.
   - The census probes each method with numbers and with control-rate
     outputs, so it never takes this branch. With an audio gate, scsynth
     runs `DemandEnvGen_next_a`, which reads `reset` sample by sample. So
     it reads the constant reset of the crate, the default, past its end.
   - No other rate method of the class library adds `K2A` or `A2K`, or
     branches on the rate of an input before `multiNew`, outside what the
     crate copies: `asAudioRateInput` of the delays, the Lag family,
     `VarLag`, `Mix`, `Splay` and `Poll`.

## Version 0.10

### The count

- `SynthDef::wire_buffers` runs `limits::check` first, so that each
  reference points back. Then it walks the UGens as fact 1 says, with the
  rules of facts 2 to 4. It gives the most buffers in use at one time.
- `WireBufferError` gives `Limit` for a definition that breaks a limit, and
  `UnknownUGen` for a UGen that the catalogue lacks.

### The catalogue

- A new field `UGenClass::cant_alias`. `generator/catalog.scd` lists
  `server/plugins` at the tag, and reads each `.cpp` file without its
  comments. It collects the names inside the two macros. If a name is not
  a core class, or a plugin names `kUnitDef_CantAliasInputsToOutputs`
  itself, the script fails.
- A new census table `audioTogether` holds `[\DemandEnvGen, \ar, [\gate,
  \reset]]`, and the generator tests it. A new field
  `Wiring::audio_together` holds the positions of those arguments.

### The fix

- In `ugen`, before the expansion: if one of the arguments of
  `audio_together` is at audio rate, each other one that is not goes
  through `K2A.ar`, in the order of the arguments.

## Tests

- `generator/fixtures.scd` writes two files.
  - `demandenvgen` holds the cases of fact 6.
  - `wirebufs` holds the cases of facts 1, 3 and 4, and one case for each
    flagged unit with an audio input: the 16 units of fact 2 other than
    `BeatTrack` and `BeatTrack2`. `PlayBuf` also comes without an audio
    input.
- `tests/golden.rs` counts 87 files. `NAMES` gains each file in its step.
  The rebuild of `DecodeB2` uses `add`, because `ugen` refuses it.
- `tests/fixtures/wire_buffers.json` holds the count of every definition of
  every golden file. `tests/wire_buffers.rs` tests the crate against it in
  CI.
- `tests/scsynth.rs` gains two tests. Each golden definition loads at a
  `-w` equal to its count, and scsynth refuses it at one less. A file with
  one definition over `-w` loads none of its definitions. With
  `SCSYNTH_SYNTHDEF_BLESS=1`, the first test then writes the fixture.
- Unit tests cover the two errors. Tests of the catalogue cover the 18
  flags and the rule of `DemandEnvGen.ar`.

## Steps

Each step is one commit on `feat/wire-buffers`. Every commit leaves the
eight commands of `CLAUDE.md` green.

1. `docs: the plan of version 0.10`.
2. `test: the golden files of 0.10, written by sclang 3.14.1`.
3. `fix: the K2A of DemandEnvGen.ar`: the table, the field, the step in
   `ugen`, and the parity of `demandenvgen`.
4. `feat: the count of wire buffers`: the flag, `wire_buffers`,
   `WireBufferError`, the parity of `wirebufs`, the tests of scsynth, and
   the fixture.
5. `docs: the release of version 0.10`.

## sc-app3

After the tag, sc-app3 takes `v0.10.0` on a branch
`build/scsynth-synthdef-0.10.0` from `origin/develop`, in a worktree of
its own. The submodule moves, and `Cargo.lock` changes one line. Core's
definitions keep their 2,301 bytes. A new test of sc-expression compiles
the markup of 16 `SinOsc` into an `Out` against `wirebufs_sixteen`, and
counts 16 wire buffers. The PR goes to `develop`, and merges once its CI
is green.

## After version 0.10

The work after version 0.10, in the order of its value for sc-app3.
`PackFFT.new` comes first, because the markup of sc-app3 reaches it with
`UnpackFFT` and expressions. The markup has no functions, which the
`pvcalc` methods take.

1. `PackFFT.new`, whose inputs follow a count, and the `pvcalc` methods of
   `PV_ChainUGen`
2. The methods of `UGen` that build graphs, such as `range`, `exprange` and
   `linlin`, which need the `signalRange` of each class, and with them
   `range`, `exprange` and `curverange` of `Env`
3. sc3-plugins
