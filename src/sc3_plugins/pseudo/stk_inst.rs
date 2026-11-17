//! The pseudo-UGens of the plugin `StkInst`: `StkBowedI` and `StkGlobals`
//! of `StkInst.sc`.

use super::multi_new;
use crate::builder::{SynthDefBuilder, symbol_codes};
use crate::error::BuildError;
use crate::pseudo::Args;
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// The controls of the bow, by their number in Stk, and the arguments of
/// `StkBowedI` that set them.
const BOW: [(f64, &str); 5] = [
    (2.0, "bowpressure"),
    (4.0, "bowposition"),
    (11.0, "vibfreq"),
    (1.0, "vibgain"),
    (128.0, "loudness"),
];

/// `StkBowedI.ar`: `StkInst.ar` of the instrument 6, `Stk.at("Bowed")`,
/// with the controls of the bow as its `args`.
pub(super) fn stk_bowed_i(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let mut controls = Vec::with_capacity(2 * BOW.len());
    for (number, name) in BOW {
        controls.push(number.into());
        controls.push(args.input(name)?);
    }
    builder.ugen(
        "StkInst",
        "ar",
        &[
            ("instNumber", 6.0.into()),
            ("freq", args.value("freq")?),
            ("gate", args.value("gate")?),
            ("onamp", args.value("onamp")?),
            ("offamp", args.value("offamp")?),
            ("args", UGenInput::Array(controls).into()),
        ],
    )
}

/// `StkGlobals.ar`: `showWarnings` and `printErrors`, then the codes of
/// `rawfilepath`, without its size.
pub(super) fn stk_globals(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let path = args
        .text("rawfilepath")?
        .ok_or_else(|| args.missing("rawfilepath"))?;
    let mut names = vec!["showWarnings", "printErrors"];
    let mut inputs = vec![args.input("showWarnings")?, args.input("printErrors")?];
    for code in symbol_codes(&path).into_iter().skip(1) {
        names.push("rawfilepath");
        inputs.push(code);
    }
    multi_new(
        builder,
        ("StkGlobals", Rate::Audio),
        &names,
        &inputs,
        &|_| Ok(1),
    )
}
