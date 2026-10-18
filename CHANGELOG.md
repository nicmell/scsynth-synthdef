# Changelog

## Unreleased

### Added

- `read_def_file`, `write_def_file`, `write_def_file_old` and
  `SynthDef::as_bytes`: definition files of versions 0, 1 and 2, as Rust
  types. Reading is strict, and both directions apply the limits of
  scsynth. See [Definition files](README.md#definition-files) and
  [Strictness](README.md#strictness).
- The feature `serde`, for the types of a definition and of a file.
