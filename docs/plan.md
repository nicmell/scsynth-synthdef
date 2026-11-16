# The plan of scsynth-synthdef

This document is the plan of version 0.15. A session starts from it and
from the README, in this folder. The plan owns its decisions and facts
until the code and the tests own them. Then the facts move to the tests,
the README and the CHANGELOG, and this file keeps only what is still open.

## What we build

Version 0.15 brings the bodies of the other pseudo-UGens of sc3-plugins.
Version 0.14 built three of them: `Greyhole`, `JPverb` and `NHHall`.

- 69 more rate methods get a body: every pseudo-UGen of sc3-plugins but
  those that the language or a missing unit rules out.
- `UGen:circleRamp` of sc3-plugins joins `send`.
- `ugen` then refuses 8 methods of sc3-plugins: `OnsetsDS.kr`, and the 7
  methods of classes without a unit.
- One file of cases holds the inputs of the golden graphs of the bodies.
  sclang builds each case and writes its bytes, and the crate builds the
  same case from the same file.

sc-app3 stays at version 0.13 until the owner asks for a move.

## Ground rules

- This is a clean implementation from the SuperCollider sources. Do not
  consult earlier compilers: neither `sc-app2/packages/synthdef-compiler`
  nor `sc-crates/crates/scsynthdef-compiler`. No search of sc-app3 reaches
  its submodule `sc-app2`.
- Where sclang writes a definition that is broken, or throws, the crate
  refuses the input. Where scsynth refuses what sclang writes, the crate
  refuses it too.
- A body adds the UGens in the order in which sclang adds them, and
  copies the bugs of sc3-plugins as sclang builds them.
- A crate item lands with its first caller, so that no commit warns about
  dead code.
- No change breaks the public API. Without the feature `sc3-plugins`, the
  crate compiles the same code and writes the same bytes.

## Decisions

The owner took these on 2026-11-16.

1. One release holds every body. sc-app3 does not move.
2. The cases of the bodies are data: `tests/fixtures/sc3_plugins/bodies.json`.
   The three reverbs of 0.14 move there.
3. Where sclang needs a number, a body refuses an output with
   `BuildError::WrongKind`, as sclang throws there.
4. `FM7` gets the rule `CheckInputs::InputCount`, and the crate builds it.
5. When the CI of the PR is green, the work stops. On the go of the owner,
   the PR merges, and `v0.15.0` becomes an annotated tag on its merge
   commit.

## Sources of truth

- The SuperCollider source at the tag `Version-3.14.1`, in
  `~/Projects/supercollider`, and sclang and scsynth 3.14.1.
- The sc3-plugins source at the tag `Version-3.14.0`, in
  `~/Projects/sc3-plugins`, and its installed folder `SC3plugins` of the
  Extensions.

## Facts established on 2026-11-16

The facts come from sclang 3.14.1 and the classes of sc3-plugins 3.14.0,
through probes of one line on stdin.

1. 30 classes outside `UGen` need a body, with 33 methods: `Phaser` has
   `ar` and `kr`, and `PulseDPW` has `ar`, `kr` and `new`. 36 methods of
   UGen classes need one too: 8 that make no instance of their class, 26
   that do more than `new1`, and both methods of `StkFlute`.
2. Every body builds the same graph at each build. `FreeVerb1` builds 124
   UGens and `Vocode` 104. `BLPF` of order 8 builds 74, `WarpOverlap` and
   `BtoUHJ` 47 each, and `HighShelf` 29.
3. Traps that the crate copies:
   - `TextVU:new1` turns a number `trig` into `Impulse.kr` before its
     default `ana` reads `trig`. So `Amplitude` always gets a
     `releaseTime` of 5. `TextVU` enters the graph before the `Amplitude`
     that it reads, and before its `A2K` at audio rate.
   - `FreeVerb1` sums its 10 taps again for each of its 8 combs, which
     gives 24 `Sum4`. Nothing reads its `OnePole`, and `optimizeGraph`
     removes it.
   - `LRLPF` lacks a semicolon before its return, and still applies `mul`
     and `add`.
   - `Dneuromodule.kr` gives `Duty.ar`, as `Dneuromodule.ar` does.
   - `StkFlute.kr` applies `madd` twice.
   - Nothing reads one `FOS` chain of `BtoUHJ`, or the `Impulse` of
     `Feedback`, and `optimizeGraph` removes them.
4. Numbers that the language computes:
   - `BHPF` and `BLPF` take their order from a table of 2, 4, 6, 8 and 10.
     Any other order throws "Message 'rate' not understood.", and so does
     an output.
   - These arguments must be numbers too: the orders of `LRHPF` and
     `LRLPF`, and `numTaps`, `numCombs` and `numAllpasses` of `FreeVerb1`.
     So must `num` of `Vocoder` and `overlaps` of `WarpOverlap`.
   - `numdims` of `SOMRd` throws outside 1 to 4. `SpectralEntropy` tests
     `numbands` against `fftsize`. The `FFTSubband` classes sort their
     `cutfreqs`.
   - Some outputs follow a count: `MatchingP` has 2 + 2n outputs,
     `NearestN` 3n, `SOMRd` numdims, and the `FFTSubband` classes n + 1.
5. The bodies of the crate compute 10 unary and 13 binary operators on
   numbers, which `tests/fixtures/arithmetic.json` checks against sclang.
   The shelves need `dbamp`, which sclang computes as `pow(10, db * 0.05)`:
   `6.dbamp` is 1.9952623149688797, and the formula gives that in f64.
   `SoftClipAmp` needs `softclip`: `(|x| - 0.25) / x` beyond 0.5. The libm
   of Linux can round `pow`, `cos` and `sin` one unit in the last place
   apart from that of macOS.
6. `Stk.at("Bowed")` is 6. `FM7` takes a control matrix of 6 rows of 3 and
   a modulation matrix of 6 rows of 6, zeros by default: 54 inputs and 6
   outputs. Its `checkInputs` asks for exactly 54 inputs. `Getenv` writes
   the size of its key, its default, and the codes of the key.
7. `Out.kr` of a demand-rate result builds in sclang.

## Version 0.15

### The cases of the bodies

- A case has a name, a call (a class, a selector, and the arguments by
  name in the order of the method), and an `Out` at `ar` or `kr`, or none.
- A value is a number, an array, `{"control": [name, default]}` (an
  argument of the graph function, with the rate of its prefix),
  `{"ugen": [class, selector, arguments]}`, `{"symbol": text}` or
  `{"bool": value}`.
- A refused case must break in sclang, and the crate must refuse it.
- `generator/sc3_plugins/fixtures.scd` writes the sclang code of each case,
  builds it, and writes `bodies.scsyndef`. `tests/sc3_plugins/bodies.rs`
  builds each case from the same JSON, and compares the bytes of each
  definition.
- `expansion` and `pv` stay in `cases.rs`, because `pv` shares one chain
  between two readers.
- Where a body computes `pow`, `sin` or `cos` of numbers, most cases take
  values whose result is exact on both platforms, such as a `dbgain` of 0
  or 20. One case per body takes other values, and CI on Linux tests it.

### The bodies

- `src/sc3_plugins/pseudo/` has one file per plugin, and JoshUGens is
  split by its class files. Each body follows the sclang code. The
  arithmetic goes through `lang::binary` and `lang::unary`, other classes
  through `builder.ugen`, and the methods of `UGen` through `send`.
- A helper adds a UGen whose outputs follow a function of its inputs:
  `FM7`, `MatchingP`, `NearestN`, `SOMRd`, the `FFTSubband` classes,
  `LPCVals`, `Dneuromodule` and `NHHall`.
- `lang` computes `dbamp` and `softclip` of numbers, and
  `generator/fixtures.scd` adds both to `arithmetic.json`.
- `generator/sc3_plugins/catalog.scd` moves the classes of each step from
  `leftOut` to `bodies`, and `argumentKinds` gains the kinds of the new
  bodies. The generator reads the n of `InputCount` from
  `FM7.numRequiredInputs`.
- `UGen:circleRamp`: the catalogue of sc3-plugins gets its methods of
  `UGen`, with `sc3_plugins::ugen_method` and `ugen_methods`. `send` finds
  a method in the core, then in sc3-plugins.

## Steps

Each step is one commit on `feature/sc3-plugins-bodies`. Every commit
leaves the eight commands of `CLAUDE.md` green.

1. `docs: the plan of version 0.15`.
2. `test: the cases of the bodies of sc3-plugins`.
3. `feat: the filters of JoshUGens and BhobUGens`.
4. `feat: the ambisonics and the UHJ of JoshUGens`.
5. `feat: the other bodies of JoshUGens`.
6. `feat: the bodies of MCLDUGens`.
7. `feat: the other bodies of sc3-plugins`.
8. `feat: UGen:circleRamp`.
9. `docs: the release of version 0.15`.

## sc-app3

sc-app3 is at version 0.13 of the crate. When the owner asks, it takes
0.14 or 0.15 by a move of its submodule `crates/scsynth-synthdef`. The
feature `sc3-plugins` stays off there. Turning it on needs a plan of
sc-app3:

- sc-expression finds a class with `catalog::lookup` and
  `catalog::pseudo_lookup`, and lists `catalog::all` in an error. It needs
  the lookups of `sc3_plugins` too.
- The scsynth that sc-app3 starts needs the plugins of sc3-plugins.
- The feature adds 258 KB to a release wasm32 build, and 28.5 KB after
  gzip.

The expression functions of the methods of `UGen`, such as
`range(x, lo, hi)`, still wait for a plan of their own in sc-app3.

## After version 0.15

Left out of sc3-plugins: ATK, whose classes come with the quark
`atk-sc3`. `CheapVerb` draws `rand` of the language at each build. `BF`,
`BFDecode`, `BFEncode` and `Reflections` compute with objects of the
language, such as `ZPoint`. `BhobTone` calls `HiShelf`, which no class
defines. `OnsetsDS` is deprecated, and gives a Function. The 7 classes
without a unit stay refused.
