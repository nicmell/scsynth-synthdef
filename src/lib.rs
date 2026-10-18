#![doc = include_str!("../README.md")]

mod builder;
mod error;
mod file;
mod limits;
mod rate;
mod synth_def;
mod topo;
mod ugen_input;

pub use builder::{Arg, ArgRate, NewUGen, SynthDefBuilder};
pub use error::{
    BuildError, CountKind, LimitError, LimitErrorKind, NameKind, ReadError, ReadErrorKind,
};
pub use file::{DefFile, Version, read_def_file, write_def_file, write_def_file_old};
pub use rate::Rate;
pub use synth_def::{ControlName, InputSpec, SynthDef, UGen, Variant};
pub use ugen_input::{OutputProxy, UGenInput};
