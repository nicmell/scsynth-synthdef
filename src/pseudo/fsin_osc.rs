//! The bodies of `FSinOsc.sc`.

use super::{Args, elements, lang};
use crate::builder::{NewUGen, SynthDefBuilder};
use crate::error::BuildError;
use crate::rate::Rate;
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

/// `Klank.ar`: the expansion of `multiNewList` over `input`, `freqscale`,
/// `freqoffset` and `decayscale`, then a `Klank` whose inputs end with the
/// triplets of the spec `[freqs, amps, times]`.
pub(super) fn klank(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let partials = triplets("Klank", &args.array("specificationsArrayRef")?, [1.0, 1.0])?;
    let values = [
        args.input("input")?,
        args.input("freqscale")?,
        args.input("freqoffset")?,
        args.input("decayscale")?,
    ];
    builder.expand(
        "Klank",
        &["input", "freqscale", "freqoffset", "decayscale"],
        &values,
        true,
        &mut |builder, instance| {
            let inputs = [instance, &partials].concat();
            builder.add(NewUGen::new("Klank", Rate::Audio).inputs(inputs))
        },
    )
}

/// `Klang.ar`: the expansion of `multiNewList` over `freqscale` and
/// `freqoffset`, then a `Klang` whose inputs end with the triplets of the
/// spec `[freqs, amps, phases]`.
pub(super) fn klang(builder: &mut SynthDefBuilder, args: &Args) -> Result<UGenInput, BuildError> {
    let partials = triplets("Klang", &args.array("specificationsArrayRef")?, [1.0, 0.0])?;
    let values = [args.input("freqscale")?, args.input("freqoffset")?];
    builder.expand(
        "Klang",
        &["freqscale", "freqoffset"],
        &values,
        true,
        &mut |builder, instance| {
            let inputs = [instance, &partials].concat();
            builder.add(NewUGen::new("Klang", Rate::Audio).inputs(inputs))
        },
    )
}

/// The triplets of `new1` of `Klank` and `Klang`:
/// `[freqs, second ?? fill(freqs.size, d0), third ?? fill(freqs.size, d1)].flop.flat`.
/// The parts wrap to the longest, as `flop` wraps them.
fn triplets(
    class: &str,
    spec: &[UGenInput],
    defaults: [f64; 2],
) -> Result<Vec<UGenInput>, BuildError> {
    let freqs = spec.first().cloned().map_or_else(Vec::new, elements);
    let mut parts = vec![freqs];
    for (index, default) in defaults.into_iter().enumerate() {
        let size = parts[0].len();
        let part = spec
            .get(index + 1)
            .cloned()
            .map_or_else(|| vec![default.into(); size], elements);
        parts.push(part);
    }
    let size = parts.iter().map(Vec::len).max().unwrap_or(0);
    if size > 0 && parts.iter().any(Vec::is_empty) {
        return Err(BuildError::EmptyInput {
            ugen: class.to_owned(),
            argument: "specificationsArrayRef".to_owned(),
        });
    }
    Ok((0..size)
        .flat_map(|index| {
            parts
                .iter()
                .map(move |part| part[index % part.len()].clone())
        })
        .collect())
}

/// Part `index` of a spec, or an array of the default, as `spec[i] ? #[x]`.
fn part(spec: &[UGenInput], index: usize, default: f64) -> UGenInput {
    spec.get(index)
        .cloned()
        .unwrap_or_else(|| UGenInput::Array(vec![default.into()]))
}
