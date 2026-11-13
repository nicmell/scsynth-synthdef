//! The pseudo-UGens of `BEQSuite.sc`.

use std::f64::consts::PI;

use super::{Args, lang};
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::ugen_input::UGenInput;

/// `BLowPass4.ar`: two `SOS` in a chain, with the coefficients of
/// `BLowPass.sc`.
pub(super) fn b_low_pass4(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    b_pass4(builder, args, false)
}

/// `BHiPass4.ar`: two `SOS` in a chain, with the coefficients of
/// `BHiPass.sc`.
pub(super) fn b_hi_pass4(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    b_pass4(builder, args, true)
}

fn b_pass4(
    builder: &mut SynthDefBuilder,
    args: &Args,
    high: bool,
) -> Result<UGenInput, BuildError> {
    let rq = lang::unary(builder, "sqrt", args.input("rq")?)?;
    let [a0, a1, a2, b1, b2] = coefficients(builder, args.input("freq")?, rq, high)?;
    let first = builder.ugen(
        "SOS",
        "ar",
        &[
            ("in", args.value("in")?),
            ("a0", a0.clone().into()),
            ("a1", a1.clone().into()),
            ("a2", a2.clone().into()),
            ("b1", b1.clone().into()),
            ("b2", b2.clone().into()),
        ],
    )?;
    builder.ugen(
        "SOS",
        "ar",
        &[
            ("in", first.into()),
            ("a0", a0.into()),
            ("a1", a1.into()),
            ("a2", a2.into()),
            ("b1", b1.into()),
            ("b2", b2.into()),
            ("mul", args.value("mul")?),
            ("add", args.value("add")?),
        ],
    )
}

/// The coefficients `[a0, a1, a0, b1, b2]` of the `sc` method of `BLowPass`,
/// or of `BHiPass` if `high`.
fn coefficients(
    builder: &mut SynthDefBuilder,
    freq: UGenInput,
    rq: UGenInput,
    high: bool,
) -> Result<[UGenInput; 5], BuildError> {
    // The unused `sr` of the method: `SampleRate` is not pure, so sclang
    // keeps it in the graph.
    builder.ugen("SampleRate", "ir", &[])?;
    let two_pi = lang::binary(builder, "*", PI.into(), 2.0.into())?;
    let cycles = lang::binary(builder, "*", two_pi, freq)?;
    let period = builder.ugen("SampleDur", "ir", &[])?;
    let w0 = lang::binary(builder, "*", cycles, period)?;
    let cos_w0 = lang::unary(builder, "cos", w0.clone())?;
    let i = lang::binary(
        builder,
        if high { "+" } else { "-" },
        1.0.into(),
        cos_w0.clone(),
    )?;
    let sine = lang::unary(builder, "sin", w0)?;
    let half = lang::binary(builder, "*", sine, 0.5.into())?;
    let alpha = lang::binary(builder, "*", half, rq)?;
    let sum = lang::binary(builder, "+", 1.0.into(), alpha.clone())?;
    let b0rz = lang::unary(builder, "reciprocal", sum)?;
    let i_half = lang::binary(builder, "*", i.clone(), 0.5.into())?;
    let a0 = lang::binary(builder, "*", i_half, b0rz.clone())?;
    let a1 = if high {
        let negated = lang::unary(builder, "neg", i)?;
        lang::binary(builder, "*", negated, b0rz.clone())?
    } else {
        lang::binary(builder, "*", i, b0rz.clone())?
    };
    let cos_twice = lang::binary(builder, "*", cos_w0, 2.0.into())?;
    let b1 = lang::binary(builder, "*", cos_twice, b0rz.clone())?;
    let difference = lang::binary(builder, "-", 1.0.into(), alpha)?;
    let negated = lang::unary(builder, "neg", b0rz)?;
    let b2 = lang::binary(builder, "*", difference, negated)?;
    Ok([a0.clone(), a1, a0, b1, b2])
}
