//! `Feedback.sc`: `Feedback`.

use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, lang};
use crate::ugen_input::UGenInput;

/// `Feedback.ar`: a `PlayBuf` of `bufnum` by `feedbackpct`, plus `in`,
/// which a `RecordBuf` writes back to `bufnum`, by `mul`. Nothing reads the
/// `Impulse` of `delayTime`.
pub(crate) fn feedback(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let freq = lang::unary(builder, "reciprocal", args.input("delayTime")?)?;
    builder.ugen("Impulse", "ar", &[("freq", freq.into())])?;
    let bufnum = args.value("bufnum")?;
    let played = builder.ugen(
        "PlayBuf",
        "ar",
        &[
            ("numChannels", 1.0.into()),
            ("bufnum", bufnum.clone()),
            ("rate", 1.0.into()),
            ("trigger", 1.0.into()),
            ("startPos", 0.0.into()),
            ("loop", 1.0.into()),
        ],
    )?;
    let scaled = lang::binary(builder, "*", played, args.input("feedbackpct")?)?;
    let delayed = lang::binary(builder, "+", scaled, args.input("in")?)?;
    builder.ugen(
        "RecordBuf",
        "ar",
        &[
            ("inputArray", delayed.clone().into()),
            ("bufnum", bufnum),
            ("offset", 0.0.into()),
            ("recLevel", 1.0.into()),
            ("preLevel", 0.0.into()),
            ("run", 1.0.into()),
            ("loop", 1.0.into()),
            ("trigger", 1.0.into()),
        ],
    )?;
    lang::binary(builder, "*", delayed, args.input("mul")?)
}
