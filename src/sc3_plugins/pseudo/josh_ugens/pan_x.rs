//! `PanX.sc`: `PanX2D`, which makes no instance of its class.

use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::Args;
use crate::ugen_input::UGenInput;

/// `PanX2D.ar` and `PanX2D.kr`: a `PanX` over `numChansX` of each channel
/// of a `PanX` over `numChansY`.
pub(crate) fn pan_x2d(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let rate = args.selector();
    let level = args.value("level")?;
    let column = builder.ugen(
        "PanX",
        rate,
        &[
            ("numChans", args.value("numChansY")?),
            ("in", args.value("in")?),
            ("pos", args.value("posY")?),
            ("level", level.clone()),
            ("width", args.value("widthY")?),
        ],
    )?;
    builder.ugen(
        "PanX",
        rate,
        &[
            ("numChans", args.value("numChansX")?),
            ("in", column.into()),
            ("pos", args.value("posX")?),
            ("level", level),
            ("width", args.value("widthX")?),
        ],
    )
}
