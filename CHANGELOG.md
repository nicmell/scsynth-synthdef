# Changelog

## 0.1.0 (2026-10-18)

The first release. It compiles synth definitions as sclang 3.14.1 compiles
them. It reads and writes their files. It carries a catalogue of the UGens
of the core class library.

### Added

- `read_def_file`, `write_def_file`, `write_def_file_old` and
  `SynthDef::as_bytes`: definition files of versions 0, 1 and 2, as Rust
  types. Reading is strict, and both directions apply the limits of
  scsynth. See [Definition files](README.md#definition-files) and
  [Strictness](README.md#strictness).
- `SynthDefBuilder`: the build of sclang without `optimizeGraph`. Controls
  from the arguments, `add`, variants, the constants and the topological
  sort of sclang. See [Building](README.md#building) and
  [Differences from sclang](README.md#differences-from-sclang).
- `SynthDefBuilder::unary_op` and `binary_op`, and the module `operator`:
  the operators of scsynth by name and by special index, generated from
  the sources of SuperCollider 3.14.1.
- The module `catalog`: the 419 UGen classes of the core class library of
  SuperCollider 3.14.1, with their arguments, their categories, the rules
  of their `checkInputs`, and the wiring of each rate method that `ugen`
  can add. See [The catalogue](README.md#the-catalogue).
- `SynthDefBuilder::ugen` and `ArgValue`: a UGen by its class name, with
  the arguments by name, the defaults and the wiring of the catalogue, and
  the checks of `checkInputs`.
- The feature `serde`, for the types of a definition and of a file.

### Known issues

- See [Left out](README.md#left-out) for what version 0.1 does not do yet,
  and [Traps](README.md#traps) for what it does not rule out.
