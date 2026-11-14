//! The bodies of `MachineListening.sc`.

use super::{Args, buf_io, lang};
use crate::builder::{NewUGen, SynthDefBuilder};
use crate::error::BuildError;
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `BeatTrack.kr`: a chain that comes from no `FFT` goes through
/// `FFT(LocalBuf(if(SampleRate.ir > 48000, 2048, 1024)), chain)` first, with
/// the arithmetic of `UGen:if`. Then a `BeatTrack` of 4 outputs for each
/// instance of the chain and `lock`.
pub(super) fn beat_track(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let mut chain = args.input("chain")?;
    if builder.class_of(&chain) != Some("FFT") {
        let rate = builder.ugen("SampleRate", "ir", &[])?;
        let high = lang::binary(builder, ">", rate, 48000.0.into())?;
        let step = lang::binary(builder, "-", 2048.0.into(), 1024.0.into())?;
        let scaled = lang::binary(builder, "*", high, step)?;
        let frames = lang::binary(builder, "+", scaled, 1024.0.into())?;
        let buffer = buf_io::new_local_buf(builder, 1.0.into(), frames)?;
        chain = builder.ugen(
            "FFT",
            "new",
            &[("buffer", buffer.into()), ("in", chain.into())],
        )?;
    }
    let values = [chain, args.input("lock")?];
    builder.expand(
        "BeatTrack",
        &["chain", "lock"],
        &values,
        true,
        &mut |builder, instance| {
            builder.add(
                NewUGen::new("BeatTrack", Rate::Control)
                    .inputs(instance.to_vec())
                    .outputs(vec![Rate::Control; 4]),
            )
        },
    )
}
