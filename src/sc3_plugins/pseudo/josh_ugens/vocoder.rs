//! `Vocoder.sc`: `Vocoder`.

use super::super::do_count;
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, float, lang};
use crate::ugen_input::UGenInput;

/// `Vocoder.ar`: `Mix.ar` of `num + 1` bands from `low` to `high`, each a
/// `BPF` of `car` by the amplitude of a `BPF` of `mod`, plus `mod` through
/// two `HPF`.
pub(crate) fn vocoder(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let num = args.constant("num")?;
    let mut bands = Vec::new();
    for index in 0..do_count(args, "num", num + 1.0)? {
        let quotient = lang::binary(builder, "/", args.input("high")?, args.input("low")?)?;
        let exponent = lang::unary(builder, "reciprocal", num.into())?;
        let ratio = lang::binary(builder, "pow", quotient, exponent)?;
        let power = lang::binary(builder, "pow", ratio, float(index).into())?;
        let cf = lang::binary(builder, "*", power, args.input("low")?)?;
        let modulator = band(builder, args, "mod", cf.clone())?;
        let tracker = builder.ugen("Amplitude", "kr", &[("in", modulator.into())])?;
        let carrier = band(builder, args, "car", cf)?;
        let product = lang::binary(builder, "*", carrier, tracker)?;
        bands.push(lang::binary(builder, "*", args.input("outscal")?, product)?);
    }
    let out = builder.ugen("Mix", "ar", &[("array", UGenInput::Array(bands).into())])?;
    let hpf = args.value("hpf")?;
    let high = builder.ugen(
        "HPF",
        "ar",
        &[("in", args.value("mod")?), ("freq", hpf.clone())],
    )?;
    let high = builder.ugen("HPF", "ar", &[("in", high.into()), ("freq", hpf)])?;
    let scaled = lang::binary(builder, "*", args.input("hpfscal")?, high)?;
    lang::binary(builder, "+", out, scaled)
}

/// `BPF.ar(signal, cf, q)` of the argument `signal`.
fn band(
    builder: &mut SynthDefBuilder,
    args: &Args,
    signal: &str,
    cf: UGenInput,
) -> Result<UGenInput, BuildError> {
    builder.ugen(
        "BPF",
        "ar",
        &[
            ("in", args.value(signal)?),
            ("freq", cf.into()),
            ("rq", args.value("q")?),
        ],
    )
}
