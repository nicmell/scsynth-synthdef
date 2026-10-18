# scsynth-synthdef

The synth definitions of SuperCollider, for Rust. This crate will compile a
graph of UGens into the
[SynthDef file format](https://doc.sccode.org/Reference/Synth-Definition-File-Format.html)
(`SCgf`), the bytes that scsynth loads with `/d_recv`. It will compile them
the way sclang compiles a `SynthDef`. It will also read those files back,
and carry a catalogue of the UGens of the core class library.

The crate follows SuperCollider 3.14.1.

## Status

The work is in progress, and nothing is implemented yet.
[The plan](https://github.com/nicmell/scsynth-synthdef/blob/main/docs/plan.md)
describes version 0.1 and its steps.

## License

Licensed under either of the Apache License, Version 2.0
(`LICENSE-APACHE`) or the MIT license (`LICENSE-MIT`), at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this crate by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
