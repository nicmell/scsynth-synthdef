//! The pseudo-UGens of `Filter.sc`.

use super::{Args, lang};
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::ugen_input::UGenInput;

/// `Changed.ar` and `Changed.kr`: `HPZ1(input).abs > threshold`.
pub(super) fn changed(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let slope = builder.ugen("HPZ1", args.selector(), &[("in", args.value("input")?)])?;
    let size = lang::unary(builder, "abs", slope)?;
    lang::binary(builder, ">", size, args.input("threshold")?)
}
