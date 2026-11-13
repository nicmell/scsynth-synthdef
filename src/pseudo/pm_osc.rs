//! The pseudo-UGens of `SCClassLibrary/backwards_compatibility/PMOsc.sc`.

use super::mix::mix;
use super::{Args, lang, number};
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::ugen_input::UGenInput;

/// `PMOsc.ar` and `PMOsc.kr`: a `SinOsc` whose phase is a `SinOsc` of
/// `modfreq`, scaled by `pmindex`.
pub(super) fn pm_osc(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let modulator = builder.ugen(
        "SinOsc",
        args.selector(),
        &[
            ("freq", args.value("modfreq")?),
            ("phase", args.value("modphase")?),
            ("mul", args.value("pmindex")?),
        ],
    )?;
    builder.ugen(
        "SinOsc",
        args.selector(),
        &[
            ("freq", args.value("carfreq")?),
            ("phase", modulator.into()),
            ("mul", args.value("mul")?),
            ("add", args.value("add")?),
        ],
    )
}

/// `MultiTap.ar`: a `RecordBuf` of `in`, then a `PlayBuf` for each time of
/// `timesArray`, scaled by its level, and the `Mix` of them, then `madd`.
pub(super) fn multi_tap(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let times = args.array("timesArray")?;
    let levels = args.array("levelsArray")?;
    let input = args.input("in")?;
    let bufnum = args.value("bufnum")?;
    builder.ugen(
        "RecordBuf",
        "ar",
        &[
            ("inputArray", input.clone().into()),
            ("bufnum", bufnum.clone()),
            ("offset", 0.0.into()),
            ("run", (-1.0).into()),
        ],
    )?;
    let rate = builder.ugen("BufSampleRate", "kr", &[("bufnum", bufnum.clone())])?;
    let channels = match &input {
        UGenInput::Array(items) => number(items.len()),
        _ => 1.0.into(),
    };
    let mut taps = Vec::with_capacity(times.len());
    for (index, time) in times.into_iter().enumerate() {
        let start = lang::binary(builder, "*", time, rate.clone())?;
        let tap = builder.ugen(
            "PlayBuf",
            "ar",
            &[
                ("numChannels", channels.clone().into()),
                ("bufnum", bufnum.clone()),
                ("rate", (-1.0).into()),
                ("trigger", 1.0.into()),
                ("startPos", start.into()),
                ("loop", 1.0.into()),
            ],
        )?;
        let level = levels.get(index).cloned().unwrap_or(1.0.into());
        taps.push(builder.madd(tap, level, 0.0)?);
    }
    let mixed = mix(builder, &taps)?;
    builder.madd(mixed, args.input("mul")?, args.input("add")?)
}
