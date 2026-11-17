//! `BiquadEQ.sc`: `HighShelf`, `LowShelf` and `Notch`, whose `ar` the
//! classes `RBJHighShelf`, `RBJLowShelf` and `RBJNotch` inherit.

use std::f64::consts::PI;

use super::normalized_sos;
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, lang};
use crate::ugen_input::UGenInput;

/// `Notch.ar` and `RBJNotch.ar`.
pub(crate) fn notch(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let (sin, cos) = sin_cos(builder, args.input("freq")?)?;
    let scaled = lang::binary(builder, "*", args.input("rq")?, sin)?;
    let alpha = lang::binary(builder, "/", scaled, 2.0.into())?;
    let a0 = lang::binary(builder, "+", 1.0.into(), alpha.clone())?;
    let a1 = lang::binary(builder, "*", (-2.0).into(), cos)?;
    let a2 = lang::binary(builder, "-", 1.0.into(), alpha)?;
    normalized_sos(
        builder,
        args.value("in")?,
        [1.0.into(), a1.clone(), 1.0.into()],
        [a0, a1, a2],
        None,
        [args.value("mul")?, args.value("add")?],
    )
}

/// `LowShelf.ar` and `RBJLowShelf.ar`.
pub(crate) fn low_shelf(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    shelf(builder, args, true)
}

/// `HighShelf.ar` and `RBJHighShelf.ar`.
pub(crate) fn high_shelf(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    shelf(builder, args, false)
}

/// The shelves: `LowShelf` if `low`, else `HighShelf`. The coefficients of
/// `HighShelf` swap the signs of the terms in `cos` and of the factors 2,
/// and `HighShelf` computes `a0.neg` once.
fn shelf(builder: &mut SynthDefBuilder, args: &Args, low: bool) -> Result<UGenInput, BuildError> {
    let a = lang::unary(builder, "dbamp", args.input("dbgain")?)?;
    let (sin, cos) = sin_cos(builder, args.input("freq")?)?;
    let beta = beta(builder, &a, args.input("shelfslope")?)?;
    let a_minus = lang::binary(builder, "-", a.clone(), 1.0.into())?;
    let a_plus = lang::binary(builder, "+", a.clone(), 1.0.into())?;
    let minus_cos = lang::binary(builder, "*", a_minus.clone(), cos.clone())?;
    let plus_cos = lang::binary(builder, "*", a_plus.clone(), cos)?;
    let bs = lang::binary(builder, "*", beta, sin)?;
    let (plus, minus, two) = if low {
        ("+", "-", 2.0)
    } else {
        ("-", "+", -2.0)
    };
    let sum = lang::binary(builder, plus, a_plus.clone(), minus_cos.clone())?;
    let a0 = lang::binary(builder, "+", sum, bs.clone())?;
    let inner = lang::binary(builder, plus, a_minus.clone(), plus_cos.clone())?;
    let a1 = lang::binary(builder, "*", (-two).into(), inner)?;
    let sum = lang::binary(builder, plus, a_plus.clone(), minus_cos.clone())?;
    let a2 = lang::binary(builder, "-", sum, bs.clone())?;
    let sum = lang::binary(builder, minus, a_plus.clone(), minus_cos.clone())?;
    let sum = lang::binary(builder, "+", sum, bs.clone())?;
    let b0 = lang::binary(builder, "*", a.clone(), sum)?;
    let scaled = lang::binary(builder, "*", two.into(), a.clone())?;
    let inner = lang::binary(builder, minus, a_minus, plus_cos)?;
    let b1 = lang::binary(builder, "*", scaled, inner)?;
    let sum = lang::binary(builder, minus, a_plus, minus_cos)?;
    let sum = lang::binary(builder, "-", sum, bs)?;
    let b2 = lang::binary(builder, "*", a, sum)?;
    let neg = if low {
        None
    } else {
        Some(lang::unary(builder, "neg", a0.clone())?)
    };
    normalized_sos(
        builder,
        args.value("in")?,
        [b0, b1, b2],
        [a0, a1, a2],
        neg.as_ref(),
        [args.value("mul")?, args.value("add")?],
    )
}

/// `a.sqrt * ((a + a.reciprocal) * (shelfslope.reciprocal - 1) + 2).sqrt`.
fn beta(
    builder: &mut SynthDefBuilder,
    a: &UGenInput,
    shelfslope: UGenInput,
) -> Result<UGenInput, BuildError> {
    let root = lang::unary(builder, "sqrt", a.clone())?;
    let reciprocal = lang::unary(builder, "reciprocal", a.clone())?;
    let sum = lang::binary(builder, "+", a.clone(), reciprocal)?;
    let slope = lang::unary(builder, "reciprocal", shelfslope)?;
    let slope = lang::binary(builder, "-", slope, 1.0.into())?;
    let product = lang::binary(builder, "*", sum, slope)?;
    let product = lang::binary(builder, "+", product, 2.0.into())?;
    let product = lang::unary(builder, "sqrt", product)?;
    lang::binary(builder, "*", root, product)
}

/// `omega = 2pi * freq/SampleRate.ir`, then `omega.sin` and `omega.cos`.
fn sin_cos(
    builder: &mut SynthDefBuilder,
    freq: UGenInput,
) -> Result<(UGenInput, UGenInput), BuildError> {
    let cycles = lang::binary(builder, "*", (2.0 * PI).into(), freq)?;
    let rate = builder.ugen("SampleRate", "ir", &[])?;
    let omega = lang::binary(builder, "/", cycles, rate)?;
    let sin = lang::unary(builder, "sin", omega.clone())?;
    Ok((sin, lang::unary(builder, "cos", omega)?))
}
