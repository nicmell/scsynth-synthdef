//! The pseudo-UGens of `BufIO.sc`.

use super::{Args, lang};
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::ugen_input::UGenInput;

/// `Tap.ar`: a `PlayBuf` that starts `delaytime` seconds back in the buffer.
pub(super) fn tap(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let bufnum = args.value("bufnum")?;
    let scale = builder.ugen("BufRateScale", "kr", &[("bufnum", bufnum.clone())])?;
    let rate = builder.ugen("SampleRate", "ir", &[])?;
    let back = lang::unary(builder, "neg", rate)?;
    let frames = lang::binary(builder, "*", back, scale.clone())?;
    let start = lang::binary(builder, "*", args.input("delaytime")?, frames)?;
    builder.ugen(
        "PlayBuf",
        "ar",
        &[
            ("numChannels", args.value("numChannels")?),
            ("bufnum", bufnum),
            ("rate", scale.into()),
            ("trigger", 0.0.into()),
            ("startPos", start.into()),
            ("loop", 1.0.into()),
        ],
    )
}
