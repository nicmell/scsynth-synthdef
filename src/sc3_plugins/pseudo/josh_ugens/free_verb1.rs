//! `FreeVerb1.sc`: `FreeVerb1`.

use super::super::do_count;
use crate::builder::{SynthDefBuilder, count_to_usize};
use crate::error::BuildError;
use crate::pseudo::{Args, elements, float, lang};
use crate::ugen_input::UGenInput;

/// The delays and levels of the early reflections.
const TAPS: [(f64, f64); 10] = [
    (0.0046, 0.841),
    (0.0218, 0.504),
    (0.0268, 0.379),
    (0.0298, 0.346),
    (0.0485, 0.272),
    (0.0572, 0.192),
    (0.0595, 0.217),
    (0.0708, 0.181),
    (0.0741, 0.142),
    (0.0797, 0.134),
];

/// The delays and levels of the combs.
const COMBS: [(f64, f64); 8] = [
    (0.025_306_1, 0.46),
    (0.026_938_8, 0.48),
    (0.028_956_9, 0.50),
    (0.030_748_3, 0.52),
    (0.032_244_9, 0.53),
    (0.033_809_5, 0.55),
    (0.035_306_1, 0.56),
    (0.036_666_7, 0.57),
];

/// The delays of the allpasses.
const ALLPASSES: [f64; 4] = [0.012_607_7, 0.01, 0.007_732_43, 0.005_102_04];

/// `FreeVerb1.ar`: the taps of a `DelayL` of `in`, then `numCombs` `CombL`
/// in a chain, each on `in` plus the sum of the taps, then `numAllpasses`
/// `AllpassN`. Nothing reads the `OnePole` of the mono mix of `in`.
pub(crate) fn free_verb1(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let input = args.input("in")?;
    let num_taps = args.constant("numTaps")?;
    let num_combs = args.constant("numCombs")?;
    let num_allpasses = args.constant("numAllpasses")?;
    let sample = builder.ugen("SampleRate", "ir", &[])?;
    let sample = lang::unary(builder, "reciprocal", sample)?;
    let mono = match &input {
        UGenInput::Array(items) if !items.is_empty() => {
            builder.ugen("Mix", "new", &[("array", input.clone().into())])?
        }
        other => other.clone(),
    };
    builder.ugen(
        "OnePole",
        "ar",
        &[("in", mono.into()), ("coef", 0.6.into())],
    )?;
    let taps = if num_taps > 0.0 {
        let count = kept(args, "numTaps", num_taps, TAPS.len())?;
        let (times, levels): (Vec<UGenInput>, Vec<UGenInput>) = TAPS[..count]
            .iter()
            .map(|&(time, level)| (time.into(), level.into()))
            .unzip();
        let delays = [
            ("in", input.clone().into()),
            ("maxdelaytime", UGenInput::Array(times.clone()).into()),
            ("delaytime", UGenInput::Array(times).into()),
            ("mul", UGenInput::Array(levels).into()),
        ];
        Some(builder.ugen("DelayL", "ar", &delays)?)
    } else {
        None
    };
    let combs = do_count(args, "numCombs", num_combs.min(float(COMBS.len())))?;
    let mut out = input.clone();
    for &(time, level) in &COMBS[..combs] {
        let taps = taps.clone().ok_or_else(|| {
            args.wrong_kind(
                "numTaps",
                "a whole number from 1 when numCombs is 1 or more",
            )
        })?;
        let sum = lang::sum(builder, elements(taps))?;
        let comb_in = lang::binary(builder, "+", input.clone(), sum)?;
        let delay = comb_delay(builder, args, time, &sample)?;
        let comb = [
            ("in", comb_in.into()),
            ("maxdelaytime", 0.1.into()),
            ("delaytime", delay.into()),
            ("decaytime", args.value("revTime")?),
            ("mul", level.into()),
            ("add", out.into()),
        ];
        out = builder.ugen("CombL", "ar", &comb)?;
    }
    if num_allpasses > 0.0 {
        let count = kept(args, "numAllpasses", num_allpasses, ALLPASSES.len())?;
        for &time in &ALLPASSES[..count] {
            let allpass = [
                ("in", out.into()),
                ("maxdelaytime", time.into()),
                ("delaytime", time.into()),
                ("decaytime", 1.0.into()),
            ];
            out = builder.ugen("AllpassN", "ar", &allpass)?;
        }
    }
    Ok(out)
}

/// The delay of a comb: `LFNoise2.ar(4100, 0.0002, 1) * (time *
/// combScale).round(sample) + (hfDamping * sample / 2)`.
fn comb_delay(
    builder: &mut SynthDefBuilder,
    args: &Args,
    time: f64,
    sample: &UGenInput,
) -> Result<UGenInput, BuildError> {
    let noise = builder.ugen(
        "LFNoise2",
        "ar",
        &[
            ("freq", 4100.0.into()),
            ("mul", 0.0002.into()),
            ("add", 1.0.into()),
        ],
    )?;
    let scaled = lang::binary(builder, "*", time.into(), args.input("combScale")?)?;
    let rounded = lang::binary(builder, "round", scaled, sample.clone())?;
    let wobble = lang::binary(builder, "*", noise, rounded)?;
    let damping = lang::binary(builder, "*", args.input("hfDamping")?, sample.clone())?;
    let damping = lang::binary(builder, "/", damping, 2.0.into())?;
    lang::binary(builder, "+", wobble, damping)
}

/// The size of `data.copyRange(0, (count - 1).min(data.size))`, for a
/// `count` above 0 and the `size` of `data`. `copyRange` fails on a Float,
/// and takes all of `data` past its end.
#[allow(clippy::float_cmp)]
fn kept(args: &Args, name: &str, count: f64, size: usize) -> Result<usize, BuildError> {
    if count.fract() != 0.0 {
        return Err(args.wrong_kind(name, "a whole number"));
    }
    Ok(count_to_usize(count.min(float(size))))
}
