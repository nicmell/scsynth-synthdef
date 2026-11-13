//! The pseudo-UGens of `SoundIn.sc`.

use super::{Args, lang, number};
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::ugen_input::UGenInput;

/// `SoundIn.ar`: `In.ar(NumOutputBuses.ir + bus)`, with one `In` of several
/// channels for consecutive buses, then `madd`.
pub(super) fn sound_in(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let offset = builder.ugen("NumOutputBuses", "ir", &[])?;
    let channels = match args.input("bus")? {
        UGenInput::Array(buses) => {
            if !buses.iter().all(numbers_only) {
                return Err(args.wrong_kind("bus", "a number, an output, or an array of numbers"));
            }
            if buses.is_empty() {
                return Err(BuildError::EmptyInput {
                    ugen: "SoundIn".to_owned(),
                    argument: "bus".to_owned(),
                });
            }
            if consecutive(&buses) {
                let first = lang::binary(builder, "+", offset, buses[0].clone())?;
                builder.ugen(
                    "In",
                    "ar",
                    &[
                        ("bus", first.into()),
                        ("numChannels", number(buses.len()).into()),
                    ],
                )?
            } else {
                let each = lang::binary(builder, "+", offset, UGenInput::Array(buses))?;
                builder.ugen("In", "ar", &[("bus", each.into())])?
            }
        }
        bus => {
            let at = lang::binary(builder, "+", offset, bus)?;
            builder.ugen(
                "In",
                "ar",
                &[("bus", at.into()), ("numChannels", 1.0.into())],
            )?
        }
    };
    builder.madd(channels, args.input("mul")?, args.input("add")?)
}

fn numbers_only(input: &UGenInput) -> bool {
    match input {
        UGenInput::Number(_) => true,
        UGenInput::Array(items) => items.iter().all(numbers_only),
        UGenInput::Output(_) => false,
    }
}

/// Whether each bus is the bus before it plus 1, as `==` compares numbers and
/// arrays in sclang.
fn consecutive(buses: &[UGenInput]) -> bool {
    buses
        .windows(2)
        .all(|pair| equal(&pair[1], &plus_one(&pair[0])))
}

fn plus_one(input: &UGenInput) -> UGenInput {
    match input {
        UGenInput::Number(number) => UGenInput::Number(number + 1.0),
        UGenInput::Array(items) => UGenInput::Array(items.iter().map(plus_one).collect()),
        UGenInput::Output(output) => UGenInput::Output(*output),
    }
}

#[allow(clippy::float_cmp)]
fn equal(a: &UGenInput, b: &UGenInput) -> bool {
    match (a, b) {
        (UGenInput::Number(x), UGenInput::Number(y)) => x == y,
        (UGenInput::Array(xs), UGenInput::Array(ys)) => {
            xs.len() == ys.len() && xs.iter().zip(ys).all(|(x, y)| equal(x, y))
        }
        _ => false,
    }
}
