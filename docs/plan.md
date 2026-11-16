# The plan of scsynth-synthdef

This document is the plan of version 0.13. A session starts from it and
from the README, in this folder. The plan owns its decisions and facts
until the code and the tests own them. Then the facts move to the tests,
the README and the CHANGELOG, and this file keeps only what is still open.

## What we build

Version 0.13 takes the findings of a review of version 0.12. On
2026-11-16, six reviewers read the crate, one for each area: the builder,
the writer, the methods of `UGen`, the other pseudo-UGens, the catalogue
and the tests. They found six bugs, and runs of the crate and of sclang
3.14.1 reproduced each one. They also found about 1,100 lines that can go
with the same bytes, and a `finish` that can run about 23% faster.

- Each bug gets a fix, with a golden case or a test.
- The simplifications and the optimizations keep the bytes, the errors
  and the public API.

Then sc-app3 takes the release by a move of its submodule.

## Ground rules

- This is a clean implementation from the SuperCollider sources. Do not
  consult earlier compilers: neither `sc-app2/packages/synthdef-compiler`
  nor `sc-crates/crates/scsynthdef-compiler`. No search of sc-app3 reaches
  its submodule `sc-app2`.
- Where sclang writes a definition that is broken, or throws, the crate
  refuses the input.
- A crate item lands with its first caller, so that no commit warns about
  dead code.
- No change breaks the public API.

## Decisions

The owner took these on 2026-11-16.

1. The fixes come first, each in a commit with its test, before any
   refactor.
2. An output carries the serial of its UGen. When the output comes back as
   an input, the builder compares that serial. A failed call never makes a
   serial come back, and `SynthDefBuilder` stays `Clone`.
3. `optimizeGraph` runs on a stack of its own.
4. `Poll.new` takes the rate of each element of `in` before the
   expansion. `Array:poll` and `Array:dpoll` make one call of `Poll` and
   of `Dpoll`, as sclang does.
5. `SplayAz` refuses a text `levelComp`, as `Splay` does.
6. A spread of `SplayAz` at scalar or demand rate keeps its error,
   `UnsupportedRate`.
7. When the CI of the PR is green, the work stops. On the go of the owner,
   the PR merges, and `v0.13.0` becomes an annotated tag on its merge
   commit. Then sc-app3 moves, and its PR merges into `develop` once its CI
   is green.

## Sources of truth

The SuperCollider source at the tag `Version-3.14.1`, in
`~/Projects/supercollider`, and sclang and scsynth 3.14.1. These files of
`SCClassLibrary/Common` hold the methods of the fixes:

- `Audio/Poll.sc` and `Collections/Array.sc` (`poll` and `dpoll`).
- `Audio/Demand.sc` (`Dpoll`) and `Audio/Splay.sc` (`LevelComp`).

## Facts established on 2026-11-16

The facts come from programs of the crate and from sclang 3.14.1, with
probes of one line on stdin.

1. A clone of `SynthDefBuilder` keeps the id of the build, and the builder
   compares nothing else for an output. So an output of the clone passes in
   the original, and `finish` panics with "index out of bounds".
2. An output made in a `wrap` closure that fails passes after the
   rollback. `finish` then gives a definition with 0 UGens, and no error.
3. A chain of 5,000 `* 2` that nothing reads aborts `finish` with "stack
   overflow", on a thread of 2 MiB. `optimize` and `dead_code` call each
   other once for each UGen of the chain. A live chain of 20,000 passes.
4. `Array:poll` and `Array:dpoll` call `Poll(trig, this, label, trigid)`
   and `Dpoll(this, label, run, trigid)` once. So an array among the
   arguments laces with the elements:
   - `[a, b].poll([Impulse.kr(1), Impulse.kr(2)])` gives 2 `Poll`. The
     crate gives 4.
   - `[a, b].poll([1, 2, 3])` gives 3 `Poll`, labelled `UGen Array [0]`,
     `[1]` and `[0]`.
   - `[d1, d2].dpoll(run: [1, 0])` gives 2 `Dpoll`. The crate gives 4.
5. `Poll.new` takes `in.asArray.collect(_.rate).unbubble` before
   `multiNewList` expands. So `Poll.new(10, [[SinOsc.ar(1),
   SinOsc.kr(2)], SinOsc.kr(3)])` polls at audio, audio and control rate.
   The crate takes the rate after the expansion, and gives audio, control
   and control.
6. A symbol as `levelComp` throws "Message 'clip' not understood." in
   `LevelComp`, for `Splay.ar`, `SplayAz.ar` and `SplayAz.kr`. The crate
   refuses it for `Splay`, but `SplayAz` takes it as 1.
7. `DelayN.ar(-0.0, 0.2, 0.2)` writes `DC.ar(0)`, with a positive zero.
   No golden file holds this case.

## Version 0.13

### The fixes

- Each UGen of a build gets a serial from a global counter, and its
  outputs carry it in place of the id of the build. If the UGen at the
  index of an output holds its serial, the output passes. An output from
  before a fork of the builder passes in both forks.
- `optimizeGraph` keeps its visits on a stack of steps, in the order of
  the recursion of sclang. The five rewrites of a `+` or a `-` sit in one
  function.
- `Poll` runs one level of `multiNewList` itself. Each row runs at the
  rate of the selector, or for `new` at the rate of its element of `in`.
  `Array:poll` hands it the whole receiver, with a label for each element.
  `Array:dpoll` sends `Dpoll` once for each row of the flop.
- One check of `levelComp` serves `Splay` and `SplayAz`.

### The simplifications and the optimizations

- One leaf adds a checked UGen with its width-first flag, and one
  `multi_new` expands. Six copies of that code go.
- The bodies of the methods of `UGen` that differ in one selector merge.
  `lang` loses its copies from before 0.12. The copies of `lag`, `wrap`
  and `if` in other bodies become messages.
- The other pseudo-UGens lose their twin bodies and their second `flop`.
- The builder, the writer and the catalogue lose their duplicates and
  dead branches, and their docs match the code again.
- `finish` moves its nodes instead of cloning them, and the sort counts
  the antecedents instead of keeping sets.
- The tests and the generators lose their duplicate helpers. The render
  of the catalogue gets one writer of fields, and `NAMES` comes from the
  fixture folder.

## Tests

- `generator/fixtures.scd` writes two files, in a section of 0.13:
  - `pollarrays`: facts 4 and 5, with arrays of `trig`, `trigid` and
    `run`, three triggers, one label, nested receivers, and `Poll.new`.
  - `audioinput`: fact 7.
- Two refused cases, for `Splay` and for `SplayAz`, must break in sclang:
  fact 6.
- `tests/golden.rs` counts 97 files, until the step of the tests drops the
  count. `tests/fixtures/wire_buffers.json` gains the two files in step 2.
- Unit tests own facts 1 to 3.
- After each step of the optimizer, 20,000 seeded random graphs give the
  bytes or the errors of version 0.12.0.

## Steps

Each step is one commit on `fix/review-0.12`. Every commit leaves the
eight commands of `CLAUDE.md` green.

1. `docs: the plan of version 0.13`.
2. `test: the golden files of 0.13, written by sclang 3.14.1`.
3. `fix: the outputs of a clone and of a failed call`: facts 1 and 2.
4. `fix: optimizeGraph on a stack of its own`: fact 3.
5. `fix: the arrays of Poll and Dpoll, and the levelComp of SplayAz`:
   facts 4 to 6.
6. `refactor: one way to add a checked UGen, and the bodies of the
   methods of UGen`.
7. `refactor: the other pseudo-UGens`.
8. `refactor: the builder, the writer and the catalogue`.
9. `perf: a faster finish`.
10. `refactor: the tests`.
11. `refactor: the generators`. Both generators run again and change no
    file.
12. `docs: the release of version 0.13`.

## sc-app3

After the tag, sc-app3 takes `v0.13.0` on a branch
`build/scsynth-synthdef-0.13.0` from `origin/develop`, in a worktree of
its own. The submodule moves, and `Cargo.lock` changes one line. Core's
definitions keep their bytes. The PR goes to `develop`, and merges once its
CI is green.

## After version 0.13

The work after version 0.13, in the order of its value for sc-app3. The
README lists the same item under "Left out".

1. sc3-plugins
