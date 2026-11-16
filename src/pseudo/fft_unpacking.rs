//! The pseudo-UGens of `FFTUnpacking.sc`.

use super::{Args, elements, lang, number};
use crate::builder::{NewUGen, SynthDefBuilder, count_to_usize};
use crate::catalog;
use crate::check_inputs;
use crate::error::BuildError;
use crate::rate::Rate;
use crate::ugen_input::UGenInput;

/// `UnpackFFT.new`: an `Unpack1FFT` of the magnitude, then of the phase, of
/// each bin from `frombin` to `tobin`, interleaved.
pub(super) fn unpack_fft(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let chain = args.value("chain")?;
    let bufsize = args.constant("bufsize")?;
    let frombin = args.constant("frombin")?;
    let upper = bufsize / 2.0;
    let tobin = match args.optional("tobin")? {
        Some(UGenInput::Number(tobin)) if tobin < upper => tobin,
        None | Some(UGenInput::Number(_)) => upper,
        Some(_) => return Err(args.wrong_kind("tobin", "a number")),
    };
    let bins = series(frombin, tobin);
    let unpack = |builder: &mut SynthDefBuilder, measure: f64| {
        builder.ugen(
            "Unpack1FFT",
            "new",
            &[
                ("chain", chain.clone()),
                ("bufsize", bufsize.into()),
                ("binindex", bins.clone().into()),
                ("whichmeasure", measure.into()),
            ],
        )
    };
    let magnitudes = elements(unpack(builder, 0.0)?);
    let phases = elements(unpack(builder, 1.0)?);
    Ok(UGenInput::Array(
        magnitudes
            .into_iter()
            .zip(phases)
            .flat_map(|(magnitude, phase)| [magnitude, phase])
            .collect(),
    ))
}

/// `PackFFT.new`: the chain, `bufsize`, `frombin`, `tobin`, `zeroothers`,
/// the size of `magsphases` and each of its values, as `multiNewList`
/// expands them. A nil `tobin` is `bufsize / 2`.
pub(super) fn pack_fft(
    builder: &mut SynthDefBuilder,
    args: &Args,
) -> Result<UGenInput, BuildError> {
    let bufsize = args.input("bufsize")?;
    let magsphases = args.array("magsphases")?;
    let tobin = match args.optional("tobin")? {
        Some(tobin) => tobin,
        None => lang::binary(builder, "/", bufsize.clone(), 2.0.into())?,
    };
    let mut names = vec!["chain", "bufsize", "frombin", "tobin", "zeroothers"];
    let mut values = vec![
        args.input("chain")?,
        bufsize,
        args.input("frombin")?,
        tobin,
        args.input("zeroothers")?,
        number(magsphases.len()),
    ];
    names.resize(values.len() + magsphases.len(), "magsphases");
    values.extend(magsphases);
    builder.expand(
        "PackFFT",
        &names,
        &values,
        true,
        &mut |builder, instance| {
            let inputs = instance.to_vec();
            let class = catalog::lookup("PackFFT");
            if let Some(class) = class {
                check_inputs::check(class.check_inputs, Rate::Control, &inputs).map_err(
                    |message| BuildError::CheckInputs {
                        class: "PackFFT".to_owned(),
                        message,
                    },
                )?;
            }
            builder.add(
                NewUGen::new("PackFFT", Rate::Control)
                    .inputs(inputs)
                    .outputs([Rate::Control])
                    .width_first(class.is_some_and(|class| class.is_width_first)),
            )
        },
    )
}

/// `(first..last)`, as `prSimpleNumberSeries` gives it for Floats: steps of
/// 1 or -1 toward `last`.
fn series(first: f64, last: f64) -> UGenInput {
    let step = if first < last { 1.0 } else { -1.0 };
    let size = count_to_usize(((last - first) / step).trunc()) + 1;
    UGenInput::Array(
        (0..size)
            .map(|index| {
                UGenInput::Number(
                    first + step * f64::from(u32::try_from(index).unwrap_or(u32::MAX)),
                )
            })
            .collect(),
    )
}
