//! The pseudo-UGens of `Compander.sc`.

use super::Args;
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::ugen_input::UGenInput;

/// `CompanderD.ar`: a `Compander` of `in` delayed by `clampTime`, under the
/// control of `in` itself, then `madd`.
pub(super) fn compander_d(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let input = args.value("in")?;
    let clamp = args.value("clampTime")?;
    let delayed = builder.ugen(
        "DelayN",
        "ar",
        &[
            ("in", input.clone()),
            ("maxdelaytime", clamp.clone()),
            ("delaytime", clamp.clone()),
        ],
    )?;
    let compressed = builder.ugen(
        "Compander",
        "ar",
        &[
            ("in", delayed.into()),
            ("control", input),
            ("thresh", args.value("thresh")?),
            ("slopeBelow", args.value("slopeBelow")?),
            ("slopeAbove", args.value("slopeAbove")?),
            ("clampTime", clamp),
            ("relaxTime", args.value("relaxTime")?),
        ],
    )?;
    builder.madd(compressed, args.input("mul")?, args.input("add")?)
}
