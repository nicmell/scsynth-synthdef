//! `SC2 Compat.sc`: `TapN` and `DelayWr`.

use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, lang};
use crate::ugen_input::UGenInput;

/// `TapN.ar`: a `PlayBuf` of `bufnum` from `delaytime` seconds before its
/// start, with `madd`.
pub(crate) fn tap_n(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let bufnum = args.value("bufnum")?;
    let rate = builder.ugen("BufSampleRate", "kr", &[("bufnum", bufnum.clone())])?;
    let frames = lang::binary(builder, "*", args.input("delaytime")?, rate)?;
    let start = lang::binary(builder, "*", frames, (-1.0).into())?;
    let played = builder.ugen(
        "PlayBuf",
        "ar",
        &[
            ("numChannels", args.value("numChannels")?),
            ("bufnum", bufnum),
            ("rate", 1.0.into()),
            ("trigger", 1.0.into()),
            ("startPos", start.into()),
            ("loop", 1.0.into()),
        ],
    )?;
    builder.madd(played, args.input("mul")?, args.input("add")?)
}

/// `DelayWr.ar`: a `RecordBuf` of `in` into `bufnum`.
pub(crate) fn delay_wr(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    builder.ugen(
        "RecordBuf",
        "ar",
        &[
            ("inputArray", args.value("in")?),
            ("bufnum", args.value("bufnum")?),
        ],
    )
}
