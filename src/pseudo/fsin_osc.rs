//! The pseudo-UGens of `FSinOsc.sc`.

use super::{Args, elements, lang};
use crate::builder::SynthDefBuilder;
use crate::error::BuildError;
use crate::ugen_input::UGenInput;

/// `DynKlank.ar` and `DynKlank.kr`: the expansion of `multiNew` over
/// `input`, `freqscale`, `freqoffset` and `decayscale`, then the sum of a
/// `Ringz` for each partial of the spec `[freqs, amps, times]`.
pub(super) fn dyn_klank(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let spec = args.array("specificationsArrayRef")?;
    let values = [
        args.input("input")?,
        args.input("freqscale")?,
        args.input("freqoffset")?,
        args.input("decayscale")?,
    ];
    let selector = args.selector();
    builder.expand(
        "DynKlank",
        &["input", "freqscale", "freqoffset", "decayscale"],
        &values,
        true,
        &mut |builder, instance| {
            let scaled = lang::binary(builder, "*", part(&spec, 0, 440.0), instance[1].clone())?;
            let freqs = lang::binary(builder, "+", scaled, instance[2].clone())?;
            let times = lang::binary(builder, "*", part(&spec, 2, 1.0), instance[3].clone())?;
            let rings = builder.ugen(
                "Ringz",
                selector,
                &[
                    ("in", instance[0].clone().into()),
                    ("freq", freqs.into()),
                    ("decaytime", times.into()),
                    ("mul", part(&spec, 1, 1.0).into()),
                ],
            )?;
            lang::sum(builder, elements(rings))
        },
    )
}

/// `DynKlang.ar` and `DynKlang.kr`: the expansion of `multiNew` over
/// `freqscale` and `freqoffset`, then the sum of a `SinOsc` for each partial
/// of the spec `[freqs, amps, phases]`.
pub(super) fn dyn_klang(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let spec = args.array("specificationsArrayRef")?;
    let values = [args.input("freqscale")?, args.input("freqoffset")?];
    let selector = args.selector();
    builder.expand(
        "DynKlang",
        &["freqscale", "freqoffset"],
        &values,
        true,
        &mut |builder, instance| {
            let scaled = lang::binary(builder, "*", part(&spec, 0, 440.0), instance[0].clone())?;
            let freqs = lang::binary(builder, "+", scaled, instance[1].clone())?;
            let sines = builder.ugen(
                "SinOsc",
                selector,
                &[
                    ("freq", freqs.into()),
                    ("phase", part(&spec, 2, 0.0).into()),
                    ("mul", part(&spec, 1, 1.0).into()),
                ],
            )?;
            lang::sum(builder, elements(sines))
        },
    )
}

/// Part `index` of a spec, or an array of the default, as `spec[i] ? #[x]`.
fn part(spec: &[UGenInput], index: usize, default: f64) -> UGenInput {
    spec.get(index)
        .cloned()
        .unwrap_or_else(|| UGenInput::Array(vec![default.into()]))
}
