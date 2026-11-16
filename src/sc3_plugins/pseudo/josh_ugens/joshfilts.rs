//! `joshfilts.sc`: `ZLPF`, `ZHPF`, `BLPF`, `BHPF`, `LRLPF` and `LRHPF`.

use std::f64::consts::PI;

use super::normalized_sos;
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, lang};
use crate::ugen_input::UGenInput;

/// `ZLPF.ar`.
pub(crate) fn zlpf(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    second_order(builder, args, false)
}

/// `ZHPF.ar`.
pub(crate) fn zhpf(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    second_order(builder, args, true)
}

/// `ZLPF.ar`, or `ZHPF.ar` if `high`: an `SOS` for each element of
/// `damping`, each on the last, and each with `mul` and `add`.
fn second_order(
    builder: &mut SynthDefBuilder,
    args: &Args,
    high: bool,
) -> Result<UGenInput, BuildError> {
    let fc = args.input("fc")?;
    let mut out = args.input("in")?;
    for df in args.array("damping")? {
        let cycles = lang::binary(builder, "*", PI.into(), fc.clone())?;
        let rate = builder.ugen("SampleRate", "ir", &[])?;
        let ratio = lang::binary(builder, "/", cycles, rate)?;
        let tan = lang::unary(builder, "tan", ratio)?;
        let c = if high {
            tan
        } else {
            lang::unary(builder, "reciprocal", tan)?
        };
        let cc = lang::unary(builder, "squared", c.clone())?;
        let twice = lang::binary(builder, "*", 2.0.into(), df)?;
        let two_d_c = lang::binary(builder, "*", twice, c)?;
        let sum = lang::binary(builder, "+", 1.0.into(), two_d_c.clone())?;
        let sum = lang::binary(builder, "+", sum, cc.clone())?;
        let b0 = lang::unary(builder, "reciprocal", sum)?;
        let two = if high { -2.0 } else { 2.0 };
        let b1 = lang::binary(builder, "*", two.into(), b0.clone())?;
        let doubled = lang::binary(builder, "*", 2.0.into(), b0.clone())?;
        let difference = if high {
            lang::binary(builder, "-", cc.clone(), 1.0.into())?
        } else {
            lang::binary(builder, "-", 1.0.into(), cc.clone())?
        };
        let a1 = lang::binary(builder, "*", doubled, difference)?;
        let sum = lang::binary(builder, "-", 1.0.into(), two_d_c)?;
        let sum = lang::binary(builder, "+", sum, cc)?;
        let a2 = lang::binary(builder, "*", b0.clone(), sum)?;
        out = normalized_sos(
            builder,
            out.into(),
            [b0.clone(), b1, b0],
            [1.0.into(), a1, a2],
            None,
            [args.value("mul")?, args.value("add")?],
        )?;
    }
    Ok(out)
}

/// `BLPF.ar`.
pub(crate) fn blpf(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    butterworth(builder, args, "ZLPF")
}

/// `BHPF.ar`.
pub(crate) fn bhpf(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    butterworth(builder, args, "ZHPF")
}

/// `BLPF.ar` or `BHPF.ar`: `filter` with the dampings of the order, then
/// `* mul + add`.
fn butterworth(
    builder: &mut SynthDefBuilder,
    args: &Args,
    filter: &str,
) -> Result<UGenInput, BuildError> {
    let damping = dampings(args.constant("order")?)
        .ok_or_else(|| args.wrong_kind("order", "2, 4, 6, 8 or 10"))?;
    let out = builder.ugen(
        filter,
        "ar",
        &[
            ("in", args.value("in")?),
            ("fc", args.value("freq")?),
            ("damping", damping.into()),
        ],
    )?;
    times_plus(builder, out, args)
}

/// `LRLPF.ar`.
pub(crate) fn lrlpf(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    linkwitz_riley(builder, args, "BLPF")
}

/// `LRHPF.ar`.
pub(crate) fn lrhpf(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    linkwitz_riley(builder, args, "BHPF")
}

/// `LRLPF.ar` or `LRHPF.ar`: `filter` twice in a chain, of half the order,
/// then `* mul + add`.
fn linkwitz_riley(
    builder: &mut SynthDefBuilder,
    args: &Args,
    filter: &str,
) -> Result<UGenInput, BuildError> {
    let half = args.constant("order")? / 2.0;
    if dampings(half).is_none() {
        return Err(args.wrong_kind("order", "4, 8, 12, 16 or 20"));
    }
    let mut out = args.input("in")?;
    for _ in 0..2 {
        out = builder.ugen(
            filter,
            "ar",
            &[
                ("in", out.into()),
                ("order", half.into()),
                ("freq", args.value("freq")?),
            ],
        )?;
    }
    times_plus(builder, out, args)
}

/// The dampings of `BLPF` and `BHPF` for an order, which `asInteger`
/// truncates, or none for an order that their table lacks.
#[allow(clippy::float_cmp)]
fn dampings(order: f64) -> Option<UGenInput> {
    let table: [(f64, &[f64]); 4] = [
        (4.0, &[0.924, 0.383]),
        (6.0, &[0.966, 0.707, 0.259]),
        (8.0, &[0.981, 0.831, 0.556, 0.195]),
        (10.0, &[0.988, 0.891, 0.707, 0.454, 0.156]),
    ];
    if order.trunc() == 2.0 {
        return Some((1.0 / 2f64.sqrt()).into());
    }
    let (_, values) = table.iter().find(|&&(key, _)| key == order.trunc())?;
    Some(UGenInput::Array(
        values.iter().map(|&value| value.into()).collect(),
    ))
}

/// `out * mul + add`, the last line of the Butterworth and Linkwitz-Riley
/// filters.
fn times_plus(
    builder: &mut SynthDefBuilder,
    out: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let product = lang::binary(builder, "*", out, args.input("mul")?)?;
    lang::binary(builder, "+", product, args.input("add")?)
}
