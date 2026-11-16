# scsynth-synthdef

A Rust crate for SuperCollider synth definitions. It compiles them into
the SynthDef file format (`SCgf`) and reads them back. It also carries a
catalogue of the core UGens, and one of sc3-plugins behind a feature.
SuperCollider 3.14.1 and sc3-plugins 3.14.0 are the reference. This file
is the operating manual.

The README is the reference of the crate. [docs/plan.md](docs/plan.md) holds
what is still open after version 0.14. Read both before any work.

## Ground rules

- This is a clean implementation from the SuperCollider sources. Do not
  consult earlier compilers: neither `sc-app2/packages/synthdef-compiler` nor
  `sc-crates/crates/scsynthdef-compiler`.
- The SuperCollider source is in `~/Projects/supercollider`, at the tag
  `Version-3.14.1`. `sclang` and `scsynth` 3.14.1 are on the PATH.
- The sc3-plugins source is in `~/Projects/sc3-plugins`, at the tag
  `Version-3.14.0`. Its plugins are in the folder `SC3plugins` of the
  Extensions.
- Never touch the scsynth on UDP 57110. A test runs its own scsynth in NRT
  mode, which opens no port.

## Commands

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features
cargo clippy --all-features --target wasm32-unknown-unknown -- -D warnings
cargo +1.85.0 test --all-features    # the MSRV
cargo deny check
```

CI (`.github/workflows/ci.yml`) runs them with `--locked`, and also
`cargo package`.

## Conventions

- Edition 2024, MSRV 1.85, clippy pedantic, no unsafe code, docs on every
  public item. CI runs clippy with `-D warnings`.
- A comment states a fact that the code cannot: a measured number, a trap, a
  constraint. Design rationale goes in `docs/`.
- One fact, one owner. The owner is the test that breaks with the fact.
- Work goes on a branch, with one commit per step of the plan. Every commit
  leaves fmt, clippy, the tests and the docs green. A branch reaches `main`
  through a GitHub PR.
