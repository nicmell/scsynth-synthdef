//! The pseudo-UGens of the plugin `SkUGens`: `FM7`.

use super::multi_new;
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::pseudo::{Args, elements};
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `FM7.ar`: six outputs at audio rate, from `ctlMatrix` and `modMatrix`
/// flattened by one level. A nil matrix is the zeros of `controlMatrix`, 6
/// rows of 3, or of `modMatrix`, 6 rows of 6. `checkInputs` asks for 54
/// inputs.
pub(super) fn fm7(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let mut names = Vec::new();
    let mut inputs = Vec::new();
    for (name, columns) in [("ctlMatrix", 3), ("modMatrix", 6)] {
        let values = match args.optional(name)? {
            Some(UGenInput::Array(rows)) => rows.into_iter().flat_map(elements).collect(),
            Some(_) => return Err(args.wrong_kind(name, "an array")),
            None => vec![UGenInput::Number(0.0); 6 * columns],
        };
        for value in values {
            names.push(name);
            inputs.push(value);
        }
    }
    multi_new(builder, ("FM7", Rate::Audio), &names, &inputs, &|_| Ok(6))
}
