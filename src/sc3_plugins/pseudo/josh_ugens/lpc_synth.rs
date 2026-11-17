//! `LPCSynth.sc`: `LPCSynth` and `LPCVals`, which clip `pointer` to 0 and 1
//! before their instance.

use super::super::multi_new;
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, lang};
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `LPCSynth.ar`: an `LPCSynth` of the clipped `pointer`, with `madd`.
pub(crate) fn lpc_synth(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let pointer = clipped(builder, args)?;
    let inputs = [args.input("buffer")?, args.input("signal")?, pointer];
    let names = ["buffer", "signal", "pointer"];
    let synth = multi_new(builder, ("LPCSynth", Rate::Audio), &names, &inputs, &|_| {
        Ok(1)
    })?;
    builder.madd(synth, args.input("mul")?, args.input("add")?)
}

/// `LPCVals.ar` and `LPCVals.kr`: an `LPCVals` of three outputs, of the
/// clipped `pointer`.
pub(crate) fn lpc_vals(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let rate = if args.selector() == "ar" {
        Rate::Audio
    } else {
        Rate::Control
    };
    let pointer = clipped(builder, args)?;
    let inputs = [args.input("buffer")?, pointer];
    multi_new(
        builder,
        ("LPCVals", rate),
        &["buffer", "pointer"],
        &inputs,
        &|_| Ok(3),
    )
}

/// `pointer.min(1).max(0)`.
fn clipped(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let below = lang::binary(builder, "min", args.input("pointer")?, 1.0.into())?;
    lang::binary(builder, "max", below, 0.0.into())
}
