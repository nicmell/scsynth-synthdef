//! The bodies of `BufIO.sc`.

use super::{Args, lang, new1};
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `LocalBuf.new`: a `LocalBuf` for each instance of `numChannels` and
/// `numFrames`, as `multiNew` expands them.
pub(super) fn local_buf(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let values = [args.input("numChannels")?, args.input("numFrames")?];
    builder.expand(
        "LocalBuf",
        &["numChannels", "numFrames"],
        &values,
        true,
        &mut |builder, instance| new_local_buf(builder, instance[0].clone(), instance[1].clone()),
    )
}

/// `LocalBuf:new1`: the `MaxLocalBufs` of the build counts one more buffer,
/// and the `LocalBuf` takes it as its last input.
pub(super) fn new_local_buf(
    builder: &mut SynthDefBuilder,
    channels: UGenInput,
    frames: UGenInput,
) -> Result<UGenInput, BuildError> {
    let max = builder.max_local_bufs()?;
    new1(
        builder,
        "LocalBuf",
        Rate::Scalar,
        vec![channels, frames, max],
    )
}

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
