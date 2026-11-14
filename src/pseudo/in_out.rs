//! The bodies of `InOut.sc`.

use super::Args;
use crate::builder::{NewUGen, SynthDefBuilder};
use crate::error::BuildError;
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `LocalIn.ar` and `LocalIn.kr`: `multiNew` takes the count and each
/// element of `default` apart. Each instance has as many outputs as its
/// count, and its defaults wrap to the count.
pub(super) fn local_in(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let rate = if args.selector() == "ar" {
        Rate::Audio
    } else {
        Rate::Control
    };
    let defaults = args.array("default")?;
    let mut names = vec!["numChannels"];
    names.resize(defaults.len() + 1, "default");
    let mut values = vec![args.input("numChannels")?];
    values.extend(defaults);
    builder.expand(
        "LocalIn",
        &names,
        &values,
        true,
        &mut |builder, instance| {
            let channels = args.count_of("numChannels", &instance[0])?;
            let defaults = &instance[1..];
            if defaults.is_empty() {
                return Err(BuildError::EmptyInput {
                    ugen: "LocalIn".to_owned(),
                    argument: "default".to_owned(),
                });
            }
            let inputs: Vec<UGenInput> = (0..channels)
                .map(|channel| defaults[channel % defaults.len()].clone())
                .collect();
            builder.add(
                NewUGen::new("LocalIn", rate)
                    .inputs(inputs)
                    .outputs(vec![rate; channels]),
            )
        },
    )
}
