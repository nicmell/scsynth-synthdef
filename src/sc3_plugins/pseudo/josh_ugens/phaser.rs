//! `Phaser.sc`: `Phaser`.

use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, lang};
use crate::ugen_input::UGenInput;

/// `Phaser.ar` and `Phaser.kr`: an `LFSaw` from `phase`, wrapped to half of
/// `mul` and raised by it, plus `add`.
pub(crate) fn phaser(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let mul = args.input("mul")?;
    let half = lang::binary(builder, "/", mul.clone(), 2.0.into())?;
    let level = lang::binary(builder, "+", mul, args.input("phase")?)?;
    let saw = builder.ugen(
        "LFSaw",
        args.selector(),
        &[
            ("freq", args.value("freq")?),
            ("iphase", half.clone().into()),
            ("mul", level.into()),
        ],
    )?;
    let wrapped = lang::binary(builder, "wrap2", saw, half.clone())?;
    let raised = lang::binary(builder, "+", wrapped, half)?;
    lang::binary(builder, "+", raised, args.input("add")?)
}
