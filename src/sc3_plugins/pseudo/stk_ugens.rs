//! The pseudo-UGens of the plugin `StkUGens`: `StkFlute` of
//! `STKclassdefs.sc`.

use super::multi_new;
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::Args;
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `StkFlute.ar` and `StkFlute.kr`: a `StkFlute` with `madd`, which `kr`
/// applies twice.
pub(super) fn stk_flute(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let names = ["freq", "jetDelay", "noisegain", "jetRatio"];
    let mut inputs = Vec::with_capacity(names.len());
    for name in names {
        inputs.push(args.input(name)?);
    }
    let rate = if args.selector() == "ar" {
        Rate::Audio
    } else {
        Rate::Control
    };
    let flute = multi_new(builder, ("StkFlute", rate), &names, &inputs, &|_| Ok(1))?;
    let (mul, add) = (args.input("mul")?, args.input("add")?);
    let flute = builder.madd(flute, mul.clone(), add.clone())?;
    if rate == Rate::Control {
        builder.madd(flute, mul, add)
    } else {
        Ok(flute)
    }
}
