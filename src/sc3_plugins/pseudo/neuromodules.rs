//! The pseudo-UGens of the plugin `Neuromodules`: `Dneuromodule`.

use super::{multi_new, output_count};
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, elements};
use crate::rate::Rate;
use crate::ugen_input::{ArgValue, UGenInput};

/// The arrays of `Dneuromodule`, in the order of its inputs.
const ARRAYS: [&str; 3] = ["theta", "x", "weights"];

/// `Dneuromodule.new`: `numChannels` outputs at demand rate, from
/// `numChannels` and the elements of the arrays. A nil array adds nothing,
/// as `++` of nil does.
pub(super) fn dneuromodule(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let mut names = vec!["numChannels"];
    let mut inputs = vec![args.input("numChannels")?];
    for name in ARRAYS {
        for element in args.optional(name)?.map(elements).unwrap_or_default() {
            names.push(name);
            inputs.push(element);
        }
    }
    let rate = ("Dneuromodule", Rate::Demand);
    multi_new(
        builder,
        rate,
        &names,
        &inputs,
        &|instance| match instance.first() {
            Some(&UGenInput::Number(count)) => output_count(args, "numChannels", count, count),
            _ => Err(args.wrong_kind("numChannels", "a number")),
        },
    )
}

/// `Dneuromodule.ar` and `Dneuromodule.kr`: `Duty.ar` of `dt`, a reset of
/// 0 and `Dneuromodule.new`. `kr` builds `Duty.ar` too, as sclang does.
pub(super) fn dneuromodule_duty(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let mut values: Vec<(&str, ArgValue)> = vec![("numChannels", args.value("numChannels")?)];
    for name in ARRAYS {
        if let Some(array) = args.optional(name)? {
            values.push((name, array.into()));
        }
    }
    let module = builder.ugen("Dneuromodule", "new", &values)?;
    builder.ugen(
        "Duty",
        "ar",
        &[
            ("dur", args.value("dt")?),
            ("reset", 0.0.into()),
            ("level", module.into()),
        ],
    )
}
