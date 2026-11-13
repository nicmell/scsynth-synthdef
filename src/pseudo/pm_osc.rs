//! The pseudo-UGens of `SCClassLibrary/backwards_compatibility/PMOsc.sc`.

use super::Args;
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::ugen_input::UGenInput;

/// `PMOsc.ar` and `PMOsc.kr`: a `SinOsc` whose phase is a `SinOsc` of
/// `modfreq`, scaled by `pmindex`.
pub(super) fn pm_osc(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let modulator = builder.ugen(
        "SinOsc",
        args.selector(),
        &[
            ("freq", args.value("modfreq")?),
            ("phase", args.value("modphase")?),
            ("mul", args.value("pmindex")?),
        ],
    )?;
    builder.ugen(
        "SinOsc",
        args.selector(),
        &[
            ("freq", args.value("carfreq")?),
            ("phase", modulator.into()),
            ("mul", args.value("mul")?),
            ("add", args.value("add")?),
        ],
    )
}
