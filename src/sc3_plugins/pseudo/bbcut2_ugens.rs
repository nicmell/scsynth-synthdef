//! The pseudo-UGens of the plugin `BBCut2UGens`: `AutoTrack`.

use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::Args;
use crate::ugen_input::UGenInput;

/// `AutoTrack.kr`: `BeatTrack.kr` of `in`, which the core now holds.
pub(super) fn auto_track(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    builder.ugen(
        "BeatTrack",
        "kr",
        &[("chain", args.value("in")?), ("lock", args.value("lock")?)],
    )
}
