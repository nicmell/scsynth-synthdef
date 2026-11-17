//! `WarpOverlap.sc`: `WarpOverlap`.

use std::f64::consts::PI;

use super::super::do_count;
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, elements, float, lang};
use crate::ugen_input::UGenInput;

/// `WarpOverlap.ar`: `overlaps` grains of `Warp1`, which a `Stepper` and an
/// `LFSaw` move through `buffer`, each under a window of a `SinOsc`, summed.
pub(crate) fn warp_overlap(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let overlaps = args.constant("overlaps")?;
    let count = do_count(args, "overlaps", overlaps)?;
    let fill = |value: &dyn Fn(f64) -> f64| -> UGenInput {
        UGenInput::Array((0..count).map(|index| value(float(index)).into()).collect())
    };
    let windowsize = args.input("windowsize")?;
    let rate = lang::unary(builder, "reciprocal", windowsize.clone())?;
    let phases = fill(&|index| (overlaps - index) / overlaps);
    let trigger = builder.ugen(
        "Impulse",
        "kr",
        &[("freq", rate.into()), ("phase", phases.into())],
    )?;
    let rate = lang::unary(builder, "reciprocal", windowsize.clone())?;
    let phases = fill(&|index| (overlaps - index) * (2.0 * PI) / overlaps + 1.5 * PI);
    let sines = builder.ugen(
        "SinOsc",
        "kr",
        &[("freq", rate.into()), ("phase", phases.into())],
    )?;
    let windows = builder.send(sines, "range", &[("lo", (-0.1).into()), ("hi", 0.9.into())])?;
    let windows = lang::binary(builder, "max", windows, 0.0.into())?;
    let counter = counter(builder, args, overlaps, count, trigger)?;
    let rate = lang::unary(builder, "reciprocal", windowsize.clone())?;
    let phases = fill(&|index| (overlaps - index) * 2.0 / overlaps + 1.0);
    let saws = builder.ugen(
        "LFSaw",
        "kr",
        &[("freq", rate.into()), ("iphase", phases.into())],
    )?;
    let span = lang::binary(builder, "/", windowsize, args.input("stretch")?)?;
    let saws = builder.send(saws, "range", &[("lo", 0.0.into()), ("hi", span.into())])?;
    let positions = lang::binary(builder, "+", counter, saws)?;
    let duration = builder.ugen("BufDur", "kr", &[("bufnum", args.value("buffer")?)])?;
    let pointers = lang::binary(builder, "/", positions, duration)?;
    let warps = builder.ugen(
        "Warp1",
        "ar",
        &[
            ("numChannels", args.value("numChans")?),
            ("bufnum", args.value("buffer")?),
            ("pointer", pointers.into()),
            ("freqScale", args.value("harm")?),
            ("windowSize", args.value("warpwindowsize")?),
            ("envbufnum", (-1.0).into()),
            ("overlaps", args.value("warpwindowoverlaps")?),
            ("windowRandRatio", args.value("warpwindowran")?),
        ],
    )?;
    let windowed = lang::binary(builder, "*", warps, windows)?;
    let sum = lang::sum(builder, elements(windowed))?;
    builder.madd(sum, args.input("mul")?, args.input("add")?)
}

/// `Stepper.kr(trigger, 0, starts + starttime, BufDur.kr(buffer),
/// windowsize * step)`, where each start but the first is `(overlaps - i) *
/// ((-1 * windowsize * step) / overlaps)`. The nil `resetval` of `Stepper`
/// takes `min`.
fn counter(
    builder: &mut SynthDefBuilder,
    args: &Args,
    overlaps: f64,
    count: usize,
    trigger: UGenInput,
) -> Result<UGenInput, BuildError> {
    let (windowsize, step) = (args.input("windowsize")?, args.input("step")?);
    let mut starts = Vec::with_capacity(count);
    for index in 0..count {
        if index == 0 {
            starts.push(0.0.into());
            continue;
        }
        let back = lang::binary(builder, "*", (-1.0).into(), windowsize.clone())?;
        let back = lang::binary(builder, "*", back, step.clone())?;
        let back = lang::binary(builder, "/", back, overlaps.into())?;
        let factor = overlaps - float(index);
        starts.push(lang::binary(builder, "*", factor.into(), back)?);
    }
    let starts = lang::binary(
        builder,
        "+",
        UGenInput::Array(starts),
        args.input("starttime")?,
    )?;
    let duration = builder.ugen("BufDur", "kr", &[("bufnum", args.value("buffer")?)])?;
    let stride = lang::binary(builder, "*", windowsize, step)?;
    builder.ugen(
        "Stepper",
        "kr",
        &[
            ("trig", trigger.into()),
            ("reset", 0.0.into()),
            ("min", starts.clone().into()),
            ("max", duration.into()),
            ("step", stride.into()),
            ("resetval", starts.into()),
        ],
    )
}
