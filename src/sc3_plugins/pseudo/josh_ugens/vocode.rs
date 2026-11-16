//! `Vocode.sc`: `Vocode` and `VocodeBand`, on a table of 19 bands.

use super::super::{first, numbers};
use crate::builder::{SynthDefBuilder, count_to_usize};
use crate::error::BuildError;
use crate::pseudo::{Args, elements, float, lang};
use crate::ugen_input::UGenInput;

/// The centers of the bands.
const FREQS: [f64; 19] = [
    50.0, 150.0, 250.0, 350.0, 450.0, 570.0, 700.0, 840.0, 1000.0, 1170.0, 1370.0, 1600.0, 1850.0,
    2150.0, 2500.0, 2900.0, 3400.0, 4000.0, 4800.0,
];

/// The widths of the bands, before `anabwscale`.
const WIDTHS: [f64; 19] = [
    50.0, 50.0, 50.0, 50.0, 60.0, 70.0, 80.0, 80.0, 100.0, 110.0, 130.0, 150.0, 180.0, 200.0,
    250.0, 400.0, 500.0, 600.0, 700.0,
];

/// `Vocode.ar`: a `Resonz` of the excitation at the pitch of `src` in each
/// band, by its amplitude there, summed.
pub(crate) fn vocode(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let widths = lang::binary(builder, "*", numbers(&WIDTHS), args.input("anabwscale")?)?;
    let rq = lang::binary(builder, "/", widths.clone(), numbers(&FREQS))?;
    let ana = builder.ugen(
        "BPF",
        "ar",
        &[
            ("in", args.value("src")?),
            ("freq", numbers(&FREQS).into()),
            ("rq", rq.into()),
        ],
    )?;
    let amps = builder.ugen("Amplitude", "ar", &[("in", ana.clone().into())])?;
    let mut freqs = Vec::with_capacity(FREQS.len());
    let bands = FREQS.iter().zip(elements(ana)).zip(elements(widths));
    for ((&freq, band), width) in bands {
        freqs.push(band_pitch(builder, args, band, freq.into(), width)?);
    }
    let excitation = excitation(builder, args)?;
    let freqs = lang::binary(
        builder,
        "*",
        UGenInput::Array(freqs),
        args.input("freqmul")?,
    )?;
    let resonz = builder.ugen(
        "Resonz",
        "ar",
        &[
            ("in", excitation.into()),
            ("freq", freqs.into()),
            ("bwr", args.value("outbw")?),
        ],
    )?;
    let weighted = lang::binary(builder, "*", resonz, amps)?;
    let sum = lang::sum(builder, elements(weighted))?;
    builder.madd(sum, args.input("mul")?, args.input("add")?)
}

/// `VocodeBand.ar`: `Vocode.ar` of the band `band` alone, whose `Resonz` is
/// at the center of the band.
pub(crate) fn vocode_band(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let widths = lang::binary(builder, "*", numbers(&WIDTHS), args.input("anabwscale")?)?;
    let band = args.input("band")?;
    let freq = pick(args, &band, &elements(numbers(&FREQS)))?;
    let width = pick(args, &band, &elements(widths))?;
    let rq = lang::binary(builder, "/", width.clone(), freq.clone())?;
    let ana = builder.ugen(
        "BPF",
        "ar",
        &[
            ("in", args.value("src")?),
            ("freq", freq.clone().into()),
            ("rq", rq.into()),
        ],
    )?;
    let amps = builder.ugen("Amplitude", "ar", &[("in", ana.clone().into())])?;
    band_pitch(builder, args, ana, freq.clone(), width)?;
    let excitation = excitation(builder, args)?;
    let center = lang::binary(builder, "*", freq, args.input("freqmul")?)?;
    let resonz = builder.ugen(
        "Resonz",
        "ar",
        &[
            ("in", excitation.into()),
            ("freq", center.into()),
            ("bwr", args.value("outbw")?),
        ],
    )?;
    let weighted = lang::binary(builder, "*", resonz, amps)?;
    builder.madd(weighted, args.input("mul")?, args.input("add")?)
}

/// `table.at(band)` of sclang, for a number or an array of numbers: `at`
/// truncates an index, gives nil past the table, and fails on an array in
/// the array. The crate refuses the nil too.
fn pick(args: &Args, band: &UGenInput, table: &[UGenInput]) -> Result<UGenInput, BuildError> {
    let at = |index: &UGenInput| match *index {
        UGenInput::Number(number) if number > -1.0 && number < float(table.len()) => {
            Ok(table[count_to_usize(number.trunc())].clone())
        }
        _ => Err(args.wrong_kind("band", "a number from 0 to 18, or an array of them")),
    };
    match band {
        UGenInput::Array(items) => items
            .iter()
            .map(at)
            .collect::<Result<_, _>>()
            .map(UGenInput::Array),
        other => at(other),
    }
}

/// `Pitch.kr(band, freq, freq - width, freq + width)[0]`.
fn band_pitch(
    builder: &mut SynthDefBuilder,
    args: &Args,
    band: UGenInput,
    freq: UGenInput,
    width: UGenInput,
) -> Result<UGenInput, BuildError> {
    let low = lang::binary(builder, "-", freq.clone(), width.clone())?;
    let high = lang::binary(builder, "+", freq.clone(), width)?;
    let pitch = builder.ugen(
        "Pitch",
        "kr",
        &[
            ("in", band.into()),
            ("initFreq", freq.into()),
            ("minFreq", low.into()),
            ("maxFreq", high.into()),
        ],
    )?;
    let [freq] = first(args, &["src"], "a signal", pitch)?;
    Ok(freq)
}

/// `harm * haspch + (noise * (1 - haspch))`, where `haspch` is the lagged
/// `hasFreq` of a `Pitch` of `src`.
fn excitation(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let pitch = builder.ugen(
        "Pitch",
        "kr",
        &[
            ("in", args.value("src")?),
            ("initFreq", 100.0.into()),
            ("minFreq", 60.0.into()),
            ("maxFreq", 440.0.into()),
        ],
    )?;
    let [_, has] = first(
        args,
        &["src"],
        "a signal, or an array of two or more",
        pitch,
    )?;
    let has = builder.ugen(
        "Lag2",
        "kr",
        &[("in", has.into()), ("lagTime", 0.01.into())],
    )?;
    let voiced = lang::binary(builder, "*", args.input("harm")?, has.clone())?;
    let rest = lang::binary(builder, "-", 1.0.into(), has)?;
    let unvoiced = lang::binary(builder, "*", args.input("noise")?, rest)?;
    lang::binary(builder, "+", voiced, unvoiced)
}
