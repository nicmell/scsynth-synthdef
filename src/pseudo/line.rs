//! The pseudo-UGens of `Line.sc`.

use super::{Args, lang};
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::ugen_input::UGenInput;

/// `LinLin.ar`, `MulAdd(in, scale, offset)`, and `LinLin.kr`,
/// `in * scale + offset`, with the scale and the offset of the ranges.
pub(super) fn lin_lin(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let span = lang::binary(builder, "-", args.input("dsthi")?, args.input("dstlo")?)?;
    let range = lang::binary(builder, "-", args.input("srchi")?, args.input("srclo")?)?;
    let scale = lang::binary(builder, "/", span, range)?;
    let shift = lang::binary(builder, "*", scale.clone(), args.input("srclo")?)?;
    let offset = lang::binary(builder, "-", args.input("dstlo")?, shift)?;
    if args.selector() == "ar" {
        return builder.madd(args.input("in")?, scale, offset);
    }
    let product = lang::binary(builder, "*", args.input("in")?, scale)?;
    lang::binary(builder, "+", product, offset)
}

/// `Silent.ar`: one `DC.ar(0)`, repeated `numChannels` times.
pub(super) fn silent(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let channels = args.count("numChannels")?;
    let silence = builder.ugen("DC", "ar", &[("in", 0.0.into())])?;
    Ok(if channels == 1 {
        silence
    } else {
        UGenInput::Array(vec![silence; channels])
    })
}
