# The plan of scsynth-synthdef

This document is the plan of version 0.14. A session starts from it and
from the README, in this folder. The plan owns its decisions and facts
until the code and the tests own them. Then the facts move to the tests,
the README and the CHANGELOG, and this file keeps only what is still open.

## What we build

Version 0.14 brings the UGens of sc3-plugins, the plugins that most
installations of SuperCollider add to the core. They get a catalogue of
their own, as the core has, in a folder of its own under `src/`.

- A cargo feature `sc3-plugins`, off by default, adds the module
  `sc3_plugins`. It holds the 491 UGen classes of sc3-plugins 3.14.0, as
  probes in sclang 3.14.1 measure them.
- `ugen` adds the 636 rate methods of these classes that a wiring
  describes, of 679. It also builds three pseudo-UGens: `Greyhole`,
  `JPverb` and `NHHall`.
- Golden files that sclang writes check each addable method byte for byte.
  scsynth checks each unit and each count of wire buffers.

Then sc-app3 takes the release by a move of its submodule. The other
pseudo-UGens of sc3-plugins come in version 0.15.

## Ground rules

- This is a clean implementation from the SuperCollider sources. Do not
  consult earlier compilers: neither `sc-app2/packages/synthdef-compiler`
  nor `sc-crates/crates/scsynthdef-compiler`. No search of sc-app3 reaches
  its submodule `sc-app2`.
- Where sclang writes a definition that is broken, or throws, the crate
  refuses the input. Where scsynth refuses what sclang writes, the crate
  refuses it too.
- A crate item lands with its first caller, so that no commit warns about
  dead code.
- No change breaks the public API. Without the feature, the crate compiles
  the same code and writes the same bytes.

## Decisions

The owner took these on 2026-11-16.

1. The reference is sc3-plugins `Version-3.14.0`, the version of the
   installed classes (facts 1 and 2). Version 3.14.1 gives the same
   catalogue.
2. The feature `sc3-plugins` is off by default, for the size of fact 11.
3. Version 0.14 brings the catalogue and three bodies. Version 0.15 brings
   the other bodies.
4. `catalog::lookup` keeps its meaning: the classes of the core. The module
   `sc3_plugins` has a lookup of its own, with the same types.
5. sc-app3 takes the move only. Turning the feature on in sc-app3 needs a
   plan of sc-app3: the lookups of sc-expression (fact 12), the plugins of
   the scsynth that sc-app3 starts, and the size of fact 11.
6. When the CI of the PR is green, the work stops. On the go of the owner,
   the PR merges, and `v0.14.0` becomes an annotated tag on its merge
   commit. Then sc-app3 moves, and its PR merges into `develop` once its CI
   is green.

## Sources of truth

- The SuperCollider source at the tag `Version-3.14.1`, in
  `~/Projects/supercollider`, and sclang and scsynth 3.14.1.
- The sc3-plugins source at the tag `Version-3.14.0`, in
  `~/Projects/sc3-plugins`. Each plugin keeps its classes in
  `source/<plugin>/sc/`, and its units in the C++ files of
  `source/<plugin>/`.
- The installed folder
  `~/Library/Application Support/SuperCollider/Extensions/SC3plugins`,
  which sclang compiles and scsynth loads.

## Facts established on 2026-11-16

The facts come from sclang and scsynth 3.14.1, and from the sources at the
tags.

1. The installed folder was built on 2026-07-16 from a clone at
   `~/Projects/supercollider/sc3-plugins`. That clone is at `6604734`,
   which is `Version-3.14.0` and 7 commits, with a one-line fix in
   `NCAnalysisUGens/SMS.cpp`. The binaries hold that path, and they are
   arm64 only, so they are not those of the release zip. The installed
   README says "Supercollider 3.14.1 Plugins" because CMake takes that
   version from the sources of SuperCollider.
2. The 226 installed class files equal those of `Version-3.14.0`, byte for
   byte. The 7 commits change no class file, no unit name and no flag.
   `Version-3.14.1` changes one class file, `vbap.sc`, and only in
   `VBAPSpeakerArray`, which builds no UGen.
3. The core library and the folder compile together without an error.
   `Main.overwriteMsg` is empty, so sc3-plugins overwrites no method. It
   adds 491 subclasses of `UGen`, where the core has 419, and 62 other
   classes. It also adds 31 methods to core classes. One of them builds a
   graph: `UGen:circleRamp`. The other 30 are helpers of the language,
   such as `SimpleNumber:gaussian`.
4. The probes of `generator/catalog.scd` find a wiring for 642 of the 679
   rate methods. They need 60 more entries of `argumentKinds`, and a probe
   that spreads a variable list of arguments, as sclang passes it
   (`LADSPA.ar`). The core has no method with such a list. 481 classes
   have rate methods, and 10 abstract classes have none. Every class has
   categories in its help. Of the 37 other methods:
   - 9 methods of 8 classes create no instance of their class:
     `FFTCentroid.kr`, `OnsetsDS.kr`, `PV_DiffMags.new`, `PanX2D.ar` and
     `kr`, `Rotate.ar`, `SoftClipAmp.ar`, `Tilt.ar` and `Tumble.ar`.
   - 28 do more than `new1` in other ways. `FeatureSave`,
     `MatchingPResynth` and the three `FFTSubband` classes put the size of
     an array before it. `MatchingP` and `NearestN` compute their outputs
     from a count. `SpectralEntropy` and `SOMRd` test numbers. `NHHall`
     takes a stereo array, and `FM7` two matrices. `DbufTag`, `Dtag` and
     `Dfsm` compute their inputs from rules. `LPCSynth` and `LPCVals` clip
     an argument. `Dneuromodule` puts a `Duty` around itself, and `TextVU`
     works as `Poll` does. `StkGlobals` and `Getenv` write codes without
     their size first.
5. 126 classes have a rule of `checkInputs` of their own. 30 of them
   belong to 26 owners outside HOA, and their rules are rules of the
   crate: `NInputs(n)`, `SameRateAsFirstInput` (`MembraneCircle`,
   `Summer`, `WrapSummer`) or `SameRateAsFirstInputIfAudio` (`EnvDetect`,
   `PermMod`, `PermModT`, `PermModArray`). Only FM7 differs: it asks for
   exactly `numRequiredInputs` inputs. The 96 class files of HOA, which
   FAUST generates, all hold the same loop: at audio rate, each of the
   first n inputs must be at audio rate.
6. No class of sc3-plugins brings an owner of `optimizeGraph`, `fftSize`
   or `convertEnv` that the core tables do not know. No rate method of
   sc3-plugins belongs to a core class. 10 classes are pure, 33 are
   width-first, 33 are PV chains, and one is unipolar: `GaussTrig`.
7. scsynth in NRT mode with `-U <plugins of the bundle>:<SC3plugins>`
   finds a unit for 466 of the 491 classes. Of the 25 others, 10 are
   abstract and 8 are the classes of fact 4 that create no instance. The
   last 7 have no unit in any plugin: `BlitB3D`, `CQ_Diff` (its source is
   `MCLD_CQ_UGens.cpp.OFF`), `Clipper32`, `FFTSubbandFlux`,
   `RMAFoodChainL`, `RegaliaMitraEQ` and `RosslerResL`. sclang writes
   them, and scsynth refuses them with "UGen 'X' not installed.". 6 of
   them have an addable method.
8. The plugins set `kUnitDef_CantAliasInputsToOutputs` in four ways. Two
   are the macros of the core, and the third is `DefineDelayCantAliasUnit`
   of `JoshUGens.cpp`. The fourth is the FAUST plugins: they pass the flag
   to `fDefineUnit` under the name of their metadata, as `GreyholeRaw`,
   `JPverbRaw` and the 96 HOA classes do. 176 of these units have a
   class. 30 have none: the 27 of ATK, and `CubicDelay`, `TALReverb` and
   `Xover2`.
9. The folder `ATK` holds the plugin `AtkUGens.scx`, with 27 units such as
   `FoaRotate`, but no class. The classes come with the quark `atk-sc3`,
   outside sc3-plugins.
10. 38 classes outside `UGen` answer `ar` or `kr`. `CheapVerb` draws
    `rand` of the language, so two builds give other bytes. `BF`,
    `BFDecode`, `BFEncode` and `Reflections` compute with objects of the
    language, such as `ZPoint` and `ThisMac.speakers`. `BhobTone` throws
    "Message 'ar' not understood.", because no class defines `HiShelf`.
    The other 32 build UGens, with the arithmetic of the language on
    numbers and outputs.
11. The core catalogue takes 102 KB of a release wasm32 build, and 17 KB
    after gzip -9: a cdylib that reads `catalog::all()`, against one that
    does not. The catalogue of sc3-plugins has about the same size.
12. sc-expression of sc-app3 finds a class with `catalog::lookup` and
    `catalog::pseudo_lookup`. It lists `catalog::all` in an error.

## Version 0.14

### The feature and the folder

- `src/sc3_plugins/` holds what belongs to sc3-plugins:
  - `mod.rs`: `lookup`, `all`, `pseudo_lookup` and `pseudo_all`, with the
    types of `catalog`: `UGenClass`, `PseudoClass` and `RateMethod`.
  - `generated.rs`: the catalogue, which `tests/render.rs` renders from
    `generator/sc3_plugins/catalog.json`.
  - `pseudo/`: the bodies of sc3-plugins and their table. They use
    `Args`, `new1` and `multi_new` of `src/pseudo/mod.rs`.
- Outside the folder, `catalog` gets a private `find` and `find_pseudo`.
  They try the core, then sc3-plugins. The calls of `catalog::lookup` and
  `catalog::pseudo_lookup` inside the crate use them. `pseudo::build`
  looks for a body in the core table, then in the table of sc3-plugins.
- The names of the two catalogues are disjoint, as sclang refuses a class
  that two files define.
- FM7 gets `CheckInputs::Custom("FM7")`, which `ugen` refuses until 0.15.

### The generator

- `generator/probes.scd` holds the functions of `catalog.scd` that
  measure a class, and the scan of the units that cannot alias. It also
  holds the tables that every class inherits from the core: the owners of
  `checkInputs`, `optimizeGraph`, `fftSize` and `convertEnv`.
  `catalog.scd` loads it, and its JSON does not change.
- `generator/sc3_plugins/sclang_conf.yaml` compiles the core library and
  the installed folder, without the other extensions and without quarks.
- `generator/sc3_plugins/catalog.scd` writes
  `generator/sc3_plugins/catalog.json`:
  - It fails unless each class comes from the core library or from the
    folder. It also fails unless each class file of the folder equals its
    blob at `Version-3.14.0` in `SC3_PLUGINS_SOURCE`, which is
    `~/Projects/sc3-plugins` by default.
  - Its tables name the 60 entries of `argumentKinds` and the 26 owners
    of `checkInputs`. For each HOA class, it measures the n of `NInputs`.
    If the rule does not hold at both rates, the script fails.
  - Its census names the 38 classes outside `UGen` and the 8 classes of
    fact 4. `Greyhole`, `JPverb` and `NHHall` have a body, and each other
    class is left out with a reason.
  - A table names the 7 classes without a unit, and their methods are not
    addable.
  - A table names the 31 methods that sc3-plugins adds to core classes,
    and why the catalogue leaves them out. The script fails on a method
    that no table names.
  - It scans the C++ sources at the tag for the four ways of fact 8, and
    skips the files that the build skips, such as `.cpp.OFF`. It fails on
    a flag set in another way, and on a unit without a class, but for the
    30 of fact 8.

### The bodies

- `Greyhole.ar` and `JPverb.ar` take `in.asArray`. Then they call
  `GreyholeRaw.ar` and `JPverbRaw.ar` on its first and its last element,
  with the arguments in the order of the raw class.
- `NHHall.ar` refuses an `in` that is not an array of two, as sclang
  throws. Then it calls `multiNew` with both elements.

## Tests

- `generator/sc3_plugins/fixtures.scd` reads the addable methods from
  `catalog.json`, and writes `tests/fixtures/sc3_plugins/`:
  - `methods.scsyndef`: one definition for each addable method, in the
    order of the catalogue. sclang calls the method itself, so the bytes
    do not depend on the wiring of the JSON. Each signal is a control at
    the rate of the method: an argument with `a_` for `ar`, `i_` for
    `ir`, and a plain one for the others. A count is 2, an array is two
    such controls, and a symbol is `/abcd`. `mul` and `add` are 0.5 and
    0.25 for a UGen with outputs. A pure class gets an `Out`, so that
    `optimizeGraph` keeps it.
  - Files of cases: arrays on a count (`PanX`, `VBAP`, `GrainBufJ`),
    array arguments (`Logger`, `NTube`, `ArrayMax`), `LADSPA`, PV chains
    with copies, and the three reverbs with mono, stereo and longer
    inputs.
  - Refused cases that must break in sclang, such as `NHHall.ar` of a mono
    input.
- `tests/sc3_plugins/main.rs` is a test target that needs the feature:
  - The crate builds each definition of `methods.scsyndef` from the
    catalogue alone, with the same values, and writes its bytes.
  - Each case file has a builder by hand, as in `tests/ugen.rs`. Each
    golden file reads, and writes back to its bytes. The counts of wire
    buffers equal `tests/fixtures/sc3_plugins/wire_buffers.json`.
- Unit tests in `src/sc3_plugins/` own these facts:
  - the counts and the order of the classes, and their names apart from
    the core
  - the 7 classes without a unit, and the units that cannot alias
  - the rules of `checkInputs`, and a body for each pseudo method
- The acceptance tests in `tests/sc3_plugins/scsynth.rs` are ignored by
  default, as those of `tests/scsynth.rs`:
  - `SCSYNTH_SC3_PLUGINS` names the folder of sc3-plugins, by default the
    installed folder. scsynth runs with
    `-U <plugins of the bundle>:<SC3plugins>`.
  - scsynth loads every golden definition of sc3-plugins. It finds a unit
    for each class with an addable method, and none for the 7.
  - Each definition loads at a `-w` equal to its count of wire buffers,
    and fails at one less. The bless writes the JSON.
  - The helpers of NRT move from `tests/scsynth.rs` to
    `tests/common/nrt.rs`.
- `tests/render.rs` renders `src/sc3_plugins/generated.rs` with the
  writers of the core classes.

## Steps

Each step is one commit on `feature/sc3-plugins`. Every commit leaves the
eight commands of `CLAUDE.md` green.

1. `docs: the plan of version 0.14`.
2. `refactor: the probes of the catalogue in a file of their own`. A run
   of `catalog.scd` changes no file.
3. `feat: the catalogue of sc3-plugins, measured by sclang 3.14.1`: facts
   1 to 8.
4. `feat: the module sc3_plugins behind the feature sc3-plugins`, with the
   three bodies.
5. `test: the golden files of sc3-plugins, written by sclang 3.14.1`.
6. `docs: the release of version 0.14`.

## sc-app3

After the tag, sc-app3 takes `v0.14.0` on a branch
`build/scsynth-synthdef-0.14.0` from `origin/develop`, in a worktree of
its own. The submodule moves, and `Cargo.lock` changes one line. Core's
definitions keep their bytes. The feature stays off. The PR goes to
`develop`, and merges once its CI is green.

## After version 0.14

Version 0.15 gets a plan of its own, with these bodies:

- The 30 other classes outside `UGen` of fact 10 that build UGens:
  - Biquad filters on `SOS`: `HighShelf`, `LowShelf`, `Notch` and the
    three RBJ classes.
  - Other filters: `ZLPF`, `ZHPF`, `BLPF`, `BHPF`, `LRLPF` and `LRHPF`.
  - Filters on `FOS`: `BhobHiShelf` and `BhobLoShelf`.
  - Reverbs: `FreeVerb1`, `BFFreeVerb` and `BFGVerb`.
  - Analysis and synthesis: `Vocode`, `VocodeBand`, `Vocoder`,
    `WarpOverlap`, `BtoUHJ` and `UHJtoB`.
  - The others: `AutoTrack`, `DelayWr`, `Feedback`, `TapN`, `Phaser`,
    `PulseDPW` and `StkBowedI`.
- The methods of UGen classes in fact 4, but `NHHall.ar`, `OnsetsDS.kr`
  and `FFTSubbandFlux.kr`. FM7 also gets a rule for its count of inputs.
- `UGen:circleRamp` for `send`.

Left out, for the reasons of facts 7, 9 and 10: `CheapVerb`, `BF`,
`BFDecode`, `BFEncode`, `Reflections`, `BhobTone`, the deprecated
`OnsetsDS`, the 7 classes without a unit, and ATK.
