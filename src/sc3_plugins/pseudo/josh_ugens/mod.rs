//! The pseudo-UGens of the plugin `JoshUGens`, by class file.

pub(super) mod biquad_eq;
pub(super) mod joshfilts;

use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::lang;
use crate::ugen_input::{ArgValue, UGenInput};

/// `SOS.ar(in, b0/a0, b1/a0, b2/a0, a1/a0.neg, a2/a0.neg, mul, add)`: the
/// last line of the filters on `SOS`. `neg` is the `a0.neg` that
/// `HighShelf` computes once before the line. The other filters compute it
/// twice in the line.
fn normalized_sos(
    builder: &mut SynthDefBuilder,
    input: ArgValue,
    [b0, b1, b2]: [UGenInput; 3],
    [a0, a1, a2]: [UGenInput; 3],
    neg: Option<&UGenInput>,
    [mul, add]: [ArgValue; 2],
) -> Result<UGenInput, BuildError> {
    let negated = |builder: &mut SynthDefBuilder| match neg {
        Some(neg) => Ok(neg.clone()),
        None => lang::unary(builder, "neg", a0.clone()),
    };
    let c0 = lang::binary(builder, "/", b0, a0.clone())?;
    let c1 = lang::binary(builder, "/", b1, a0.clone())?;
    let c2 = lang::binary(builder, "/", b2, a0.clone())?;
    let neg1 = negated(builder)?;
    let d1 = lang::binary(builder, "/", a1, neg1)?;
    let neg2 = negated(builder)?;
    let d2 = lang::binary(builder, "/", a2, neg2)?;
    builder.ugen(
        "SOS",
        "ar",
        &[
            ("in", input),
            ("a0", c0.into()),
            ("a1", c1.into()),
            ("a2", c2.into()),
            ("b1", d1.into()),
            ("b2", d2.into()),
            ("mul", mul),
            ("add", add),
        ],
    )
}
