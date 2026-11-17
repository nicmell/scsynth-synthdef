//! The methods of `UGen` that the plugin `VBAPUGens` adds: `circleRamp` of
//! `CircleRamp.sc`.

use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{self, Args, lang};
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `UGen:circleRamp`: a `CircleRamp` at the rate of the receiver, of the
/// receiver, `lagTime`, `circmin` and `circmax`.
pub(super) fn circle_ramp(
    builder: &mut SynthDefBuilder,
    receiver: UGenInput,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let rate = lang::rate(&receiver).unwrap_or(Rate::Scalar);
    let inputs = [
        receiver,
        args.input("lagTime")?,
        args.input("circmin")?,
        args.input("circmax")?,
    ];
    let names = ["in", "lagTime", "circmin", "circmax"];
    pseudo::multi_new(builder, "CircleRamp", rate, &names, &inputs)
}
