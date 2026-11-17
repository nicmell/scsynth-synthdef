//! The pseudo-UGens of the plugin `BerlachUGens`: `SoftClipAmp` of
//! `OSNonlinear.sc`.

use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, lang};
use crate::ugen_input::UGenInput;

/// `SoftClipAmp.ar`: `(in * pregain).softclip`, with `madd`. It makes no
/// instance of its class.
pub(super) fn soft_clip_amp(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let product = lang::binary(builder, "*", args.input("in")?, args.input("pregain")?)?;
    let clipped = lang::unary(builder, "softclip", product)?;
    builder.madd(clipped, args.input("mul")?, args.input("add")?)
}
