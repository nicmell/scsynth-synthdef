//! The pseudo-UGens of `Osc.sc`.

use super::mix::mix;
use super::{Args, lang, number};
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::ugen_input::UGenInput;

/// `SelectX.ar` and `SelectX.kr`: an `XFade2` of the two `Select`s around
/// `which`. The method takes `wrap`, and `new1` ignores it.
pub(super) fn select_x(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let selector = args.selector();
    let which = args.input("which")?;
    let array = args.value("array")?;
    let rounded = lang::binary(builder, "round", which.clone(), 2.0.into())?;
    let first = builder.ugen(
        "Select",
        selector,
        &[("which", rounded.into()), ("array", array.clone())],
    )?;
    let truncated = lang::binary(builder, "trunc", which.clone(), 2.0.into())?;
    let next = lang::binary(builder, "+", truncated, 1.0.into())?;
    let second = builder.ugen(
        "Select",
        selector,
        &[("which", next.into()), ("array", array)],
    )?;
    let doubled = lang::binary(builder, "*", which, 2.0.into())?;
    let centered = lang::binary(builder, "-", doubled, 1.0.into())?;
    let pan = lang::binary(builder, "fold2", centered, 1.0.into())?;
    builder.ugen(
        "XFade2",
        selector,
        &[
            ("inA", first.into()),
            ("inB", second.into()),
            ("pan", pan.into()),
        ],
    )
}

/// `SelectXFocus.new`, `.ar` and `.kr`: each input weighted by
/// `(1 - distance * focus).max(0)`, where `distance` is `absdif(which, i)`,
/// or `moddif(which, i, size)` when `wrap`, then `Mix`.
pub(super) fn select_x_focus(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let which = args.input("which")?;
    let array = args.array("array")?;
    let focus = args.input("focus")?;
    let wrap = args.boolean("wrap")?;
    let size = number(array.len());
    let mut weighted = Vec::with_capacity(array.len());
    for (index, input) in array.into_iter().enumerate() {
        let distance = if wrap {
            lang::moddif(builder, which.clone(), number(index), size.clone())?
        } else {
            lang::binary(builder, "absdif", which.clone(), number(index))?
        };
        let scaled = lang::binary(builder, "*", distance, focus.clone())?;
        let weight = lang::binary(builder, "-", 1.0.into(), scaled)?;
        let clipped = lang::binary(builder, "max", weight, 0.0.into())?;
        weighted.push(lang::binary(builder, "*", clipped, input)?);
    }
    mix(builder, &weighted)
}

/// `TChoose.ar` and `TChoose.kr`: a `Select` of `array` by a `TIRand` from 0
/// to its last index.
pub(super) fn t_choose(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let array = args.array("array")?;
    if array.is_empty() {
        return Err(BuildError::EmptyInput {
            ugen: "TChoose".to_owned(),
            argument: "array".to_owned(),
        });
    }
    let index = builder.ugen(
        "TIRand",
        args.selector(),
        &[
            ("lo", 0.0.into()),
            ("hi", number(array.len() - 1).into()),
            ("trig", args.value("trig")?),
        ],
    )?;
    builder.ugen(
        "Select",
        args.selector(),
        &[("which", index.into()), ("array", array.into())],
    )
}

/// `TWChoose.ar` and `TWChoose.kr`: a `Select` of `array` by a `TWindex` of
/// `weights`.
pub(super) fn tw_choose(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let index = builder.ugen(
        "TWindex",
        args.selector(),
        &[
            ("in", args.value("trig")?),
            ("array", args.value("weights")?),
            ("normalize", args.value("normalize")?),
        ],
    )?;
    builder.ugen(
        "Select",
        args.selector(),
        &[("which", index.into()), ("array", args.value("array")?)],
    )
}
