//! The pseudo-UGens of `Demand.sc`.

use super::Args;
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::ugen_input::UGenInput;

/// `Dstutter.new`: a `Ddup`.
pub(super) fn dstutter(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    builder.ugen(
        "Ddup",
        "new",
        &[("n", args.value("n")?), ("in", args.value("in")?)],
    )
}
