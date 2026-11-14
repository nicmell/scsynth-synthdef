//! The bodies of `DelayWr.sc`.

use super::{Args, elements, lang, number};
use crate::builder::{SynthDefBuilder, count_to_usize};
use crate::error::BuildError;
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `PingPong.ar`: a delay through a buffer, whose channels come back
/// through `LocalIn` and `LocalOut`, rotated by `rotate`, as feedback. The
/// call gives back the delayed channels.
pub(super) fn ping_pong(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let bufnum = args.input("bufnum")?;
    let inputs = match args.input("inputs")? {
        UGenInput::Array(inputs) if !inputs.is_empty() => inputs,
        _ => return Err(args.wrong_kind("inputs", "an array of channels")),
    };
    let delay_time = args.input("delayTime")?;
    let feedback = args.input("feedback")?;
    let rotate = args.constant("rotate")?;
    if rotate.fract() != 0.0 {
        return Err(args.wrong_kind("rotate", "a whole number"));
    }
    let channels = number(inputs.len());
    let sample_rate = builder.ugen("SampleRate", "ir", &[])?;
    let samples = lang::binary(builder, "*", delay_time, sample_rate)?;
    let control_dur = builder.ugen("ControlDur", "ir", &[])?;
    let late = lang::binary(builder, "-", samples, control_dur)?;
    let positive = lang::binary(builder, "max", 0.0.into(), late)?;
    let delay_samps = lang::binary(builder, "round", positive, 1.0.into())?;
    let frames = builder.ugen("BufFrames", "kr", &[("bufnum", bufnum.clone().into())])?;
    let phase = builder.ugen(
        "Phasor",
        "ar",
        &[
            ("trig", 0.0.into()),
            ("rate", 1.0.into()),
            ("start", 0.0.into()),
            ("end", frames.clone().into()),
        ],
    )?;
    let back = builder.ugen("LocalIn", "ar", &[("numChannels", channels.clone().into())])?;
    let feedback_channels = lang::binary(builder, "*", back, feedback)?;
    let behind = lang::binary(builder, "-", phase.clone(), delay_samps)?;
    let read = wrap(builder, behind, 0.0.into(), frames)?;
    let delayed = builder.ugen(
        "BufRd",
        "ar",
        &[
            ("numChannels", channels.into()),
            ("bufnum", bufnum.clone().into()),
            ("phase", read.into()),
            ("loop", 0.0.into()),
        ],
    )?;
    builder.ugen(
        "LocalOut",
        "ar",
        &[("channelsArray", delayed.clone().into())],
    )?;
    let mixed = lang::binary(builder, "+", UGenInput::Array(inputs), feedback_channels)?;
    let rotated = UGenInput::Array(rotated(elements(mixed), rotate));
    let first = elements(delayed.clone()).swap_remove(0);
    let written = lang::binary(builder, "firstArg", rotated, first)?;
    builder.ugen(
        "BufWr",
        "ar",
        &[
            ("inputArray", written.into()),
            ("bufnum", bufnum.into()),
            ("phase", phase.into()),
            ("loop", 1.0.into()),
        ],
    )?;
    Ok(delayed)
}

/// `UGen:wrap(lo, hi)`: a `Wrap` at the rate of `input`.
fn wrap(
    builder: &mut SynthDefBuilder,
    input: UGenInput,
    lo: UGenInput,
    hi: UGenInput,
) -> Result<UGenInput, BuildError> {
    let rate = lang::rate(&input).unwrap_or(Rate::Scalar);
    builder.ugen(
        "Wrap",
        lang::selector_for_rate(rate),
        &[("in", input.into()), ("lo", lo.into()), ("hi", hi.into())],
    )
}

/// `Array:rotate(n)`: element i moves to `sc_mod(i + n, size)`.
fn rotated(mut items: Vec<UGenInput>, n: f64) -> Vec<UGenInput> {
    let size = items.len();
    if size > 0 {
        let size_f64 = f64::from(u32::try_from(size).unwrap_or(u32::MAX));
        items.rotate_right(count_to_usize(n.rem_euclid(size_f64)));
    }
    items
}
