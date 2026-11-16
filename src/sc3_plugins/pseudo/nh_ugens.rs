//! The pseudo-UGens of the plugin `NHUGens`: `NHHall.sc`.

use super::multi_new;
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::Args;
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// The arguments of `NHHall.ar` after `in`, in the order of its inputs.
const ARGUMENTS: [&str; 10] = [
    "rt60",
    "stereo",
    "lowFreq",
    "lowRatio",
    "hiFreq",
    "hiRatio",
    "earlyDiffusion",
    "lateDiffusion",
    "modRate",
    "modDepth",
];

/// `NHHall.ar`: `multiNew` of the two channels of `in` and the other
/// arguments, for a UGen of two outputs. sclang throws for an `in` that is
/// not an array of two.
pub(super) fn nh_hall(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let [left, right] = <[UGenInput; 2]>::try_from(args.array("in")?)
        .map_err(|_| args.wrong_kind("in", "an array of two channels"))?;
    let mut names = vec!["in", "in"];
    let mut inputs = vec![left, right];
    for name in ARGUMENTS {
        names.push(name);
        inputs.push(args.input(name)?);
    }
    multi_new(builder, ("NHHall", Rate::Audio), &names, &inputs, 2)
}
