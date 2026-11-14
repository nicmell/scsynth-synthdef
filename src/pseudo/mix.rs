//! The pseudo-UGens of `Mix.sc`.

use super::basic_ops::{sum3_new, sum4_new};
use super::{Args, lang};
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `Mix.new`.
pub(super) fn mix_new(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    mix(builder, &args.array("array")?)
}

/// `Mix.ar`: the mix, with a `K2A` after a control-rate mix and a `DC` after
/// a number.
pub(super) fn mix_ar(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let result = mix(builder, &args.array("array")?)?;
    match lang::rate(&result) {
        Some(Rate::Audio) => Ok(result),
        Some(Rate::Control) => builder.ugen("K2A", "ar", &[("in", result.into())]),
        Some(Rate::Scalar) => builder.ugen("DC", "ar", &[("in", result.into())]),
        rate => Err(unsupported("Mix.ar", rate)),
    }
}

/// `Mix.kr`: an `A2K` before each audio-rate input, the mix, and a `DC` after
/// a number.
pub(super) fn mix_kr(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let mut array = args.array("array")?;
    if lang::rate(&UGenInput::Array(array.clone())) == Some(Rate::Audio) {
        array = array
            .into_iter()
            .map(|unit| {
                if lang::rate(&unit) == Some(Rate::Audio) {
                    builder.ugen("A2K", "kr", &[("in", unit.into())])
                } else {
                    Ok(unit)
                }
            })
            .collect::<Result<_, _>>()?;
    }
    let result = mix(builder, &array)?;
    match lang::rate(&result) {
        Some(Rate::Control) => Ok(result),
        Some(Rate::Scalar) => builder.ugen("DC", "kr", &[("in", result.into())]),
        rate => Err(unsupported("Mix.kr", rate)),
    }
}

/// `NumChannels.ar`: `input` in `numChannels` clumps, each mixed, or cut to
/// its first channel when `mixdown` is false. A single input repeats.
pub(super) fn num_channels(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let input = args.input("input")?;
    let channels = args.count("numChannels")?;
    let mixdown = args.boolean("mixdown")?;
    let single = match input {
        UGenInput::Array(items) if items.len() > 1 => {
            let mut clumps = Vec::new();
            for clump in items.chunks(items.len().div_ceil(channels)) {
                clumps.push(if clump.len() > 1 && mixdown {
                    mix(builder, clump)?
                } else {
                    clump[0].clone()
                });
            }
            return Ok(UGenInput::Array(clumps));
        }
        UGenInput::Array(items) => {
            items
                .into_iter()
                .next()
                .ok_or_else(|| BuildError::EmptyInput {
                    ugen: "NumChannels".to_owned(),
                    argument: "input".to_owned(),
                })?
        }
        other => other,
    };
    Ok(if channels == 1 {
        single
    } else {
        UGenInput::Array(vec![single; channels])
    })
}

/// `Mix.new` of an array: a `Sum4`, a `Sum3` or the `sum` of each clump of
/// 4, and again until fewer than four mixes are left.
pub(super) fn mix(
    builder: &mut SynthDefBuilder,
    array: &[UGenInput],
) -> Result<UGenInput, BuildError> {
    let mut mixed = Vec::with_capacity(array.len().div_ceil(4));
    for clump in array.chunks(4) {
        mixed.push(if let Ok(four) = <&[UGenInput; 4]>::try_from(clump) {
            sum4_new(builder, four)?
        } else if let Ok(three) = <&[UGenInput; 3]>::try_from(clump) {
            sum3_new(builder, three)?
        } else {
            lang::sum(builder, clump.to_vec())?
        });
    }
    if let Ok(three) = <&[UGenInput; 3]>::try_from(mixed.as_slice()) {
        return sum3_new(builder, three);
    }
    if mixed.len() < 3 {
        lang::sum(builder, mixed)
    } else {
        mix(builder, &mixed)
    }
}

/// The error of `Mix.ar` or `Mix.kr` for a mix at `rate`, or for a mix
/// without a rate.
fn unsupported(ugen: &str, rate: Option<Rate>) -> BuildError {
    match rate {
        Some(rate) => BuildError::UnsupportedRate {
            ugen: ugen.to_owned(),
            rate,
        },
        None => BuildError::EmptyInput {
            ugen: ugen.to_owned(),
            argument: "array".to_owned(),
        },
    }
}
