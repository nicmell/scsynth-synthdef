//! The pseudo-UGens of the plugin `BhobUGens`: `BhobLoShelf` and
//! `BhobHiShelf` of `EQ.sc`.

use std::f64::consts::PI;

use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, lang};
use crate::ugen_input::UGenInput;

/// `BhobLoShelf.ar`.
pub(super) fn lo_shelf(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    shelf(builder, args, -1.0)
}

/// `BhobHiShelf.ar`.
pub(super) fn hi_shelf(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    shelf(builder, args, 1.0)
}

/// `BhobLoShelf.ar`, or `BhobHiShelf.ar` for a `sign` of 1: an allpass
/// `FOS` whose `mul` is `sign`, mixed with `in`.
fn shelf(builder: &mut SynthDefBuilder, args: &Args, sign: f64) -> Result<UGenInput, BuildError> {
    let input = args.input("in")?;
    let scaled = lang::binary(builder, "*", PI.into(), args.input("freq")?)?;
    let period = builder.ugen("SampleDur", "ir", &[])?;
    let wc = lang::binary(builder, "*", scaled, period)?;
    let below = lang::binary(builder, "-", 1.0.into(), wc.clone())?;
    let above = lang::binary(builder, "+", 1.0.into(), wc)?;
    let a0 = lang::binary(builder, "/", below, above)?;
    let negated = lang::unary(builder, "neg", a0.clone())?;
    let allpass = builder.ugen(
        "FOS",
        "ar",
        &[
            ("in", input.clone().into()),
            ("a0", negated.into()),
            ("a1", 1.0.into()),
            ("b1", a0.into()),
            ("mul", sign.into()),
        ],
    )?;
    let sum = lang::binary(builder, "+", input.clone(), allpass.clone())?;
    let difference = lang::binary(builder, "-", input, allpass)?;
    let product = lang::binary(builder, "*", args.input("amp")?, difference)?;
    let sum = lang::binary(builder, "+", sum, product)?;
    lang::binary(builder, "*", 0.5.into(), sum)
}
